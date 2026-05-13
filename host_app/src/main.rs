use eframe::egui;
use std::error::Error;
use std::str;
use std::sync::{Arc, Mutex};
use tokio::net::UdpSocket;

mod grid;
use grid::OccupancyGrid;

#[derive(Debug)]
struct SlamPayload {
    base_x: f32,
    base_y: f32,
    heading: f32,
    scan_angle: f32,
    scan_dist: f32,
}

fn parse_payload(raw: &str) -> Option<SlamPayload> {
    let mut parts = raw.split(',');
    let payload = SlamPayload {
        base_x: parts.next()?.trim().parse().ok()?,
        base_y: parts.next()?.trim().parse().ok()?,
        heading: parts.next()?.trim().parse().ok()?,
        scan_angle: parts.next()?.trim().parse().ok()?,
        scan_dist: parts.next()?.trim().parse().ok()?,
    };
    if parts.next().is_some() { return None; }
    Some(payload)
}

// Data shared between the UDP Listener and the UI Renderer
struct RoverDashboard {
    grid: Arc<Mutex<OccupancyGrid>>,
    robot_pose: Arc<Mutex<(f32, f32, f32)>>, // (x, y, heading)
}

impl eframe::App for RoverDashboard {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request constant repaints to maintain 60 FPS
        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {
            // --- 1. ANALYTICS & METRICS PANEL ---
            let mut free_cells_count = 0;
            let mut obstacle_cells_count = 0;
            
            let grid_lock = self.grid.lock().unwrap();
            for val in grid_lock.cells.values() {
                if *val < 0.0 { free_cells_count += 1; }
                if *val > 0.0 { obstacle_cells_count += 1; }
            }

            // Each cell is 5cm x 5cm = 25 cm2 (0.0025 m2)
            let free_area_m2 = (free_cells_count as f32) * 0.0025;

            ui.heading("2D Topographical Radar - Telemetry Dashboard");
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(format!("Discovered Free Area: {:.2} m²", free_area_m2));
                ui.label(format!(" | Detected Obstacle Cells: {}", obstacle_cells_count));
            });
            ui.separator();

            // --- 2. RADAR RENDER CANVAS ---
            let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::hover());
            let center = response.rect.center(); // Center of the screen
            let scale = 2.0; // Zoom level

            // Draw the Occupancy Grid
            for (&(grid_x, grid_y), &val) in grid_lock.cells.iter() {
                let color = if val > 0.0 {
                    egui::Color32::RED // Obstacle
                } else if val < 0.0 {
                    egui::Color32::WHITE // Free Space
                } else {
                    continue; // Unknown space (leave background black)
                };

                // Convert World coordinates to Screen pixels
                let x_pos = center.x + (grid_x as f32 * 5.0 * scale);
                let y_pos = center.y - (grid_y as f32 * 5.0 * scale); // Invert Y-axis for screen coords

                let rect = egui::Rect::from_center_size(
                    egui::pos2(x_pos, y_pos),
                    egui::vec2(5.0 * scale, 5.0 * scale),
                );
                painter.rect_filled(rect, 0.0, color);
            }

            // Draw the Robot Center and Heading Vector
            let pose_lock = self.robot_pose.lock().unwrap();
            let robot_x = center.x + (pose_lock.0 * scale);
            let robot_y = center.y - (pose_lock.1 * scale);
            let heading_rad = pose_lock.2.to_radians();

            // Heading vector line
            let line_end_x = robot_x + (20.0 * heading_rad.cos());
            let line_end_y = robot_y - (20.0 * heading_rad.sin());

            painter.circle_filled(egui::pos2(robot_x, robot_y), 5.0, egui::Color32::LIGHT_BLUE);
            painter.line_segment(
                [egui::pos2(robot_x, robot_y), egui::pos2(line_end_x, line_end_y)],
                egui::Stroke::new(2.0, egui::Color32::YELLOW),
            );
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Shared data using Arc<Mutex> so both UDP and UI threads can access them safely
    let shared_grid = Arc::new(Mutex::new(OccupancyGrid::new()));
    let shared_pose = Arc::new(Mutex::new((0.0, 0.0, 0.0)));

    let grid_for_udp = shared_grid.clone();
    let pose_for_udp = shared_pose.clone();

    // Background Task: UDP Listener
    tokio::spawn(async move {
        let socket = UdpSocket::bind("0.0.0.0:4210").await.unwrap();
        let mut buf = [0u8; 1024];
        
        println!("✅ Listening for high-speed rover telemetry on UDP port 4210...");

        loop {
            if let Ok((len, src)) = socket.recv_from(&mut buf).await {
                if let Ok(raw_str) = str::from_utf8(&buf[..len]) {
                    if let Some(payload) = parse_payload(raw_str) {
                        let global_angle_rad = (payload.heading + payload.scan_angle).to_radians();
                        let ping_global_x = payload.base_x + (payload.scan_dist * global_angle_rad.cos());
                        let ping_global_y = payload.base_y + (payload.scan_dist * global_angle_rad.sin());

                        // Update Grid
                        {
                            let mut map = grid_for_udp.lock().unwrap();
                            map.process_ping(payload.base_x, payload.base_y, ping_global_x, ping_global_y);
                        }

                        // Update Robot Pose for UI
                        {
                            let mut pose = pose_for_udp.lock().unwrap();
                            *pose = (payload.base_x, payload.base_y, payload.heading);
                        }
                        
                        println!("[{}] Packet processed. Grid size: {}", src, grid_for_udp.lock().unwrap().cells.len());
                    } else {
                        eprintln!("⚠️ Malformed payload from {}: '{}'", src, raw_str.trim());
                    }
                } else {
                    eprintln!("⚠️ Received non-UTF8 packet from {}", src);
                }
            }
        }
    });

    // Main Thread: Start the GUI App
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("Autonomous Radar Interface"),
        ..Default::default()
    };

    eframe::run_native(
        "Radar Canvas",
        options,
        Box::new(|_cc| {
            Box::new(RoverDashboard {
                grid: shared_grid,
                robot_pose: shared_pose,
            })
        }),
    )?;

    Ok(())
}