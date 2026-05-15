use eframe::egui;
use std::error::Error;
use std::str;
use std::sync::{Arc, Mutex};
use std::net::UdpSocket;
use std::thread;

// Our new modules
mod network;
mod maps;
mod grid;
mod app;

use network::parse_payload;
use maps::{generate_map, MapType};
use grid::OccupancyGrid;
use app::RoverDashboard;

fn main() -> Result<(), Box<dyn Error>> {
    
    // --- MAP SELECTION ---
    // We define the starting map here so we can pass it to both the grid and the UI
    let initial_map = MapType::Hourglass; 
    let ground_truth = generate_map(initial_map); 
    // ---------------------

    // Initialize the shared state with our chosen map
    let shared_grid = Arc::new(Mutex::new(OccupancyGrid::new(ground_truth)));
    
    // Create clones for our background threads
    let grid_for_udp = shared_grid.clone();
    let grid_for_planner = shared_grid.clone();

    // THREAD 2: Sensor Input Thread (The Eyes)
    // Listens for UDP packets and updates the occupancy grid cells
    thread::spawn(move || {
        let socket = UdpSocket::bind("0.0.0.0:4210").expect("Failed to bind UDP");
        let mut buf = [0u8; 1024];
        
        loop {
            if let Ok((len, _)) = socket.recv_from(&mut buf) {
                if let Ok(raw_str) = str::from_utf8(&buf[..len]) {
                    if let Some(payload) = parse_payload(raw_str) {
                        if let Ok(mut map) = grid_for_udp.lock() {
                            // Extract pose first to satisfy the borrow checker
                            let (px, py, heading) = map.pose;
                            map.process_ping(px, py, heading, payload.scan_angle, payload.scan_dist);
                        }
                    } 
                } 
            }
        }
    });

    // THREAD 3: Autonomous Engine Control Unit (The Brain)
    // Handles the State Machine: DRIVE (following path) or THINK (A* planning)
    thread::spawn(move || {
        loop {
            thread::sleep(std::time::Duration::from_millis(50)); 
            
            let snapshot = {
                if let Ok(mut map) = grid_for_planner.lock() {
                    
                    // --- STATE: DRIVE ---
                    if let Some(next_node) = map.current_path.first().copied() {
                        let target_x = (next_node.0 as f32 * grid::CELL_SIZE) + (grid::CELL_SIZE / 2.0);
                        let target_y = (next_node.1 as f32 * grid::CELL_SIZE) + (grid::CELL_SIZE / 2.0);

                        let dx = target_x - map.pose.0;
                        let dy = target_y - map.pose.1;
                        let dist = (dx * dx + dy * dy).sqrt();

                        let speed = 1.0; 

                        if dist <= speed {
                            map.pose.0 = target_x;
                            map.pose.1 = target_y;
                            map.current_path.remove(0); 
                        } else {
                            let heading_rad = dy.atan2(dx);
                            map.pose.0 += speed * heading_rad.cos();
                            map.pose.1 += speed * heading_rad.sin();
                            map.pose.2 = heading_rad.to_degrees(); 
                        }
                        None 
                        
                    // --- STATE: THINK ---
                    } else {
                        // Capture the world state to run A* without holding the lock
                        Some((map.pose.0, map.pose.1, map.cells.clone(), map.inflated_cells.clone()))
                    }
                } else {
                    None
                }
            };

            // Calculate path if we are in the THINK state
            if let Some((sx, sy, cells, inflated)) = snapshot {
                let frontiers = OccupancyGrid::find_frontiers(&cells, &inflated);
                
                // Fallback engine: finds the best REACHABLE path
                let path = OccupancyGrid::get_reachable_path(sx, sy, &frontiers, &cells, &inflated);

                if let Ok(mut map) = grid_for_planner.lock() {
                    map.frontiers = frontiers;
                    map.current_path = path;
                }
            }
        }
    });

    // THREAD 1: UI Event Loop (The Face)
    // eframe takes over the main thread to talk to Wayland
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Radar Canvas",
        options,
        Box::new(move |_cc| Box::new(RoverDashboard { 
            grid: shared_grid, 
            selected_map: initial_map // Start the UI on the same map as the grid
        })),
    )?;

    Ok(())
}