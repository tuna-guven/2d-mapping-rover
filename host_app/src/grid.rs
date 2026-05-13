use std::collections::HashMap;

const L_OCC: f32 = 0.9;   
const L_FREE: f32 = -0.4; 
const L_MAX: f32 = 5.0;   
const L_MIN: f32 = -5.0;  
const CELL_SIZE: f32 = 5.0;

pub struct OccupancyGrid {
    pub cells: HashMap<(i32, i32), f32>,
}

impl OccupancyGrid {
    pub fn new() -> Self {
        Self { cells: HashMap::new() }
    }

    fn world_to_grid(x: f32, y: f32) -> (i32, i32) {
        ((x / CELL_SIZE).floor() as i32, (y / CELL_SIZE).floor() as i32)
    }

    fn update_cell(&mut self, grid_idx: (i32, i32), value: f32) {
        let current = self.cells.entry(grid_idx).or_insert(0.0);
        *current += value;
        if *current > L_MAX { *current = L_MAX; }
        if *current < L_MIN { *current = L_MIN; }
    }

    pub fn process_ping(&mut self, base_x: f32, base_y: f32, ping_global_x: f32, ping_global_y: f32) {
        let start_idx = Self::world_to_grid(base_x, base_y);
        let end_idx = Self::world_to_grid(ping_global_x, ping_global_y);
        let ray_cells = bresenham_line(start_idx.0, start_idx.1, end_idx.0, end_idx.1);

        for i in 0..ray_cells.len().saturating_sub(1) {
            self.update_cell(ray_cells[i], L_FREE);
        }
        self.update_cell(end_idx, L_OCC);
    }
}

fn bresenham_line(mut x0: i32, mut y0: i32, x1: i32, y1: i32) -> Vec<(i32, i32)> {
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