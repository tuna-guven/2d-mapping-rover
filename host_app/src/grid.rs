use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

const L_OCC: f32 = 0.9;
const L_FREE: f32 = -0.4;
const L_MAX: f32 = 5.0;
const L_MIN: f32 = -5.0;
pub const CELL_SIZE: f32 = 5.0; 
const INFLATION_RADIUS: i32 = 2; 

#[derive(Copy, Clone, Eq, PartialEq)]
struct Node {
    cost: i32,
    position: (i32, i32),
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost).then_with(|| self.position.cmp(&other.position))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct OccupancyGrid {
    pub cells: HashMap<(i32, i32), f32>,
    pub ground_truth: HashSet<(i32, i32)>,
    pub inflated_cells: HashSet<(i32, i32)>,
    pub frontiers: HashSet<(i32, i32)>,
    pub current_path: Vec<(i32, i32)>,
    pub pose: (f32, f32, f32), 
    pub safe_spawns: Vec<(i32, i32)>, // NEW: Pre-calculated safe zones!
}

impl OccupancyGrid {

    // The "Hard Reset" - Re-calculates spawns and clears everything for a new map
    pub fn reset_with_map(&mut self, ground_truth: HashSet<(i32, i32)>) {
        // 1. Clear state
        self.cells.clear();
        self.inflated_cells.clear();
        self.frontiers.clear();
        self.current_path.clear();
        self.ground_truth = ground_truth;

        // 2. Re-calculate safe spawns for the NEW map
        let mut interior_cells = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = vec![(0, 0)];
        visited.insert((0, 0));

        while let Some(curr) = queue.pop() {
            interior_cells.push(curr);
            let neighbors = [(curr.0 + 1, curr.1), (curr.0 - 1, curr.1), (curr.0, curr.1 + 1), (curr.0, curr.1 - 1)];
            for &n in &neighbors {
                if !visited.contains(&n) && !self.ground_truth.contains(&n) {
                    if n.0 > -60 && n.0 < 60 && n.1 > -60 && n.1 < 60 {
                        visited.insert(n);
                        queue.push(n);
                    }
                }
            }
        }

        self.safe_spawns.clear();
        for &cell in &interior_cells {
            let mut is_safe = true;
            for dx in -INFLATION_RADIUS..=INFLATION_RADIUS {
                for dy in -INFLATION_RADIUS..=INFLATION_RADIUS {
                    if self.ground_truth.contains(&(cell.0 + dx, cell.1 + dy)) {
                        is_safe = false;
                    }
                }
            }
            if is_safe { self.safe_spawns.push(cell); }
        }
        
        // 3. Teleport to a safe spot in the new map
        self.teleport_random();
    }

    pub fn teleport_random(&mut self) {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos() as usize;

        if !self.safe_spawns.is_empty() {
            let cell = self.safe_spawns[nanos % self.safe_spawns.len()];
            let heading = (nanos % 360) as f32 - 180.0;
            self.pose = (
                (cell.0 as f32 * CELL_SIZE) + (CELL_SIZE / 2.0),
                (cell.1 as f32 * CELL_SIZE) + (CELL_SIZE / 2.0),
                heading
            );
        }
    }
    
    pub fn new(ground_truth: HashSet<(i32, i32)>) -> Self {
        
        // --- 1. Flood Fill to find all interior cells ---
        let mut interior_cells = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = vec![(0, 0)]; // Start flooding from the origin
        visited.insert((0, 0));

        while let Some(curr) = queue.pop() {
            interior_cells.push(curr);
            let neighbors = [
                (curr.0 + 1, curr.1), (curr.0 - 1, curr.1), 
                (curr.0, curr.1 + 1), (curr.0, curr.1 - 1)
            ];
            for &n in &neighbors {
                if !visited.contains(&n) && !ground_truth.contains(&n) {
                    // Boundary constraint so open maps don't flood to infinity
                    if n.0 > -50 && n.0 < 50 && n.1 > -50 && n.1 < 50 {
                        visited.insert(n);
                        queue.push(n);
                    }
                }
            }
        }

        // --- 2. Filter out cells too close to walls ---
        let mut safe_spawns = Vec::new();
        for &cell in &interior_cells {
            let mut is_safe = true;
            // Check inflation radius to guarantee the 15cm rover fits!
            for dx in -INFLATION_RADIUS..=INFLATION_RADIUS {
                for dy in -INFLATION_RADIUS..=INFLATION_RADIUS {
                    if ground_truth.contains(&(cell.0 + dx, cell.1 + dy)) {
                        is_safe = false;
                    }
                }
            }
            if is_safe {
                safe_spawns.push(cell);
            }
        }

        Self {
            cells: HashMap::new(),
            ground_truth,
            inflated_cells: HashSet::new(),
            frontiers: HashSet::new(),
            current_path: Vec::new(),
            pose: (0.0, 0.0, 0.0),
            safe_spawns, // Store them for instant teleportation
        }
    }

    pub fn world_to_grid(x: f32, y: f32) -> (i32, i32) {
        ((x / CELL_SIZE).floor() as i32, (y / CELL_SIZE).floor() as i32)
    }

    fn update_cell(&mut self, grid_idx: (i32, i32), value: f32) {
        let current = self.cells.entry(grid_idx).or_insert(0.0);
        *current += value;
        if *current > L_MAX { *current = L_MAX; }
        if *current < L_MIN { *current = L_MIN; }

        if *current > 0.0 {
            for dx in -INFLATION_RADIUS..=INFLATION_RADIUS {
                for dy in -INFLATION_RADIUS..=INFLATION_RADIUS {
                    if dx * dx + dy * dy <= INFLATION_RADIUS * INFLATION_RADIUS {
                        self.inflated_cells.insert((grid_idx.0 + dx, grid_idx.1 + dy));
                    }
                }
            }
        }
    }

    pub fn process_ping(&mut self, base_x: f32, base_y: f32, heading: f32, scan_angle: f32, max_dist: f32) {
        let safe_dist = max_dist.clamp(0.0, 250.0);
        let global_angle_rad = (heading + scan_angle).to_radians();
        let ping_max_x = base_x + (safe_dist * global_angle_rad.cos());
        let ping_max_y = base_y + (safe_dist * global_angle_rad.sin());

        let start_idx = Self::world_to_grid(base_x, base_y);
        let end_idx = Self::world_to_grid(ping_max_x, ping_max_y);
        let ray_cells = bresenham_line(start_idx.0, start_idx.1, end_idx.0, end_idx.1);

        for &cell in &ray_cells {
            let mut hit_wall = false;
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if self.ground_truth.contains(&(cell.0 + dx, cell.1 + dy)) {
                        hit_wall = true;
                        break;
                    }
                }
                if hit_wall { break; }
            }

            if hit_wall {
                self.update_cell(cell, L_OCC);
                break;
            } else {
                self.update_cell(cell, L_FREE);
            }
        }
    }

    pub fn find_frontiers(cells: &HashMap<(i32, i32), f32>, inflated_cells: &HashSet<(i32, i32)>) -> HashSet<(i32, i32)> {
        let mut raw_frontiers = HashSet::new();
        for (&(x, y), &val) in cells {
            if val < 0.0 && !inflated_cells.contains(&(x, y)) {
                let neighbors = [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)];
                for n in &neighbors {
                    if !cells.contains_key(n) {
                        raw_frontiers.insert((x, y));
                        break;
                    }
                }
            }
        }

        let mut clean_frontiers = HashSet::new();
        for &f in &raw_frontiers {
            let neighbors = [(f.0 + 1, f.1), (f.0 - 1, f.1), (f.0, f.1 + 1), (f.0, f.1 - 1)];
            for n in &neighbors {
                if raw_frontiers.contains(n) {
                    clean_frontiers.insert(f);
                    break;
                }
            }
        }
        clean_frontiers
    }

    pub fn get_reachable_path(
        start_x: f32, start_y: f32, 
        frontiers: &HashSet<(i32, i32)>, 
        cells: &HashMap<(i32, i32), f32>, 
        inflated_cells: &HashSet<(i32, i32)>
    ) -> Vec<(i32, i32)> {
        let start_grid = Self::world_to_grid(start_x, start_y);
        let mut valid_frontiers = Vec::new();

        for &f in frontiers {
            let dx = (f.0 - start_grid.0) as f32;
            let dy = (f.1 - start_grid.1) as f32;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < 5.0 { continue; } // Ignore fragments right next to the wheels

            valid_frontiers.push((dist, f));
        }

        // Sort by closest first. Greedy exploration guarantees it maps the current room 
        // entirely before trying to cross the map through narrow doorways!
        valid_frontiers.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // Try every single frontier until A* successfully finds a safe, unblocked route.
        for (_, target) in valid_frontiers {
            let path = Self::find_path(start_x, start_y, target, cells, inflated_cells);
            if !path.is_empty() {
                return path; // Success!
            }
        }
        
        Vec::new() // Absolutely no frontiers are reachable. Safe to shut down.
    }

    pub fn find_path(start_x: f32, start_y: f32, goal: (i32, i32), cells: &HashMap<(i32, i32), f32>, inflated_cells: &HashSet<(i32, i32)>) -> Vec<(i32, i32)> {
        let start = Self::world_to_grid(start_x, start_y);
        if start == goal { return vec![start]; }

        let mut open_set = BinaryHeap::new();
        let mut came_from: HashMap<(i32, i32), (i32, i32)> = HashMap::new();
        let mut g_score: HashMap<(i32, i32), i32> = HashMap::new();
        let mut closed_set: HashSet<(i32, i32)> = HashSet::new();

        g_score.insert(start, 0);
        open_set.push(Node { cost: 0, position: start });

        let neighbors = [
            (0, 1, 10), (1, 0, 10), (0, -1, 10), (-1, 0, 10),
            (1, 1, 14), (1, -1, 14), (-1, 1, 14), (-1, -1, 14),
        ];

        let mut iterations = 0;
        const MAX_ITERATIONS: u32 = 50000;

        while let Some(current) = open_set.pop() {
            iterations += 1;
            if iterations > MAX_ITERATIONS { return Vec::new(); }

            if !closed_set.insert(current.position) { continue; }

            if current.position == goal {
                let mut path = Vec::new();
                let mut curr = goal;
                while curr != start {
                    path.push(curr);
                    if let Some(&next) = came_from.get(&curr) { curr = next; } else { break; }
                }
                path.reverse();
                return path;
            }

            for &(dx, dy, move_cost) in &neighbors {
                let neighbor = (current.position.0 + dx, current.position.1 + dy);

                if inflated_cells.contains(&neighbor) { continue; }
                if neighbor != start && neighbor != goal && *cells.get(&neighbor).unwrap_or(&0.0) >= 0.0 { continue; }

                let tentative_g = g_score.get(&current.position).unwrap_or(&i32::MAX).saturating_add(move_cost);

                if tentative_g < *g_score.get(&neighbor).unwrap_or(&i32::MAX) {
                    came_from.insert(neighbor, current.position);
                    g_score.insert(neighbor, tentative_g);
                    let h = ((neighbor.0 - goal.0).abs() + (neighbor.1 - goal.1).abs()) * 10;
                    open_set.push(Node { cost: tentative_g + h, position: neighbor });
                }
            }
        }
        Vec::new()
    }
}

pub fn bresenham_line(mut x0: i32, mut y0: i32, x1: i32, y1: i32) -> Vec<(i32, i32)> {
    let mut points = Vec::new();
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        points.push((x0, y0));
        if x0 == x1 && y0 == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x0 += sx; }
        if e2 <= dx { err += dx; y0 += sy; }
    }
    points
}