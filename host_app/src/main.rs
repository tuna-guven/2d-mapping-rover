use eframe::egui;
use std::error::Error;
use std::str;
use std::sync::{Arc, Mutex};
use std::net::UdpSocket;
use std::thread;

mod network;
mod maps;
mod grid;
mod app;

use network::parse_payload;
use maps::{generate_map, MapType};
use grid::OccupancyGrid;
use app::RoverDashboard;

fn main() -> Result<(), Box<dyn Error>> {
    // Sabit test için boş bir harita (veya Rooms) başlat
    let initial_map = MapType::Rooms; 
    let ground_truth = generate_map(initial_map); 

    let shared_grid = Arc::new(Mutex::new(OccupancyGrid::new(ground_truth)));
    let grid_for_udp = shared_grid.clone();

    // THREAD 2: Gözler (Python Köprüsünden UDP dinler)
    thread::spawn(move || {
        let socket = UdpSocket::bind("0.0.0.0:4210").expect("Failed to bind UDP");
        let mut buf = [0u8; 1024];
        
        loop {
            if let Ok((len, _)) = socket.recv_from(&mut buf) {
                if let Ok(raw_str) = str::from_utf8(&buf[..len]) {
                    if let Some(payload) = parse_payload(raw_str) {
                        if let Ok(mut map) = grid_for_udp.lock() {
                            // 1. Radar masada sabit, konumu hep (0,0)
                            map.pose.0 = 0.0;
                            map.pose.1 = 0.0;
                            
                            // 2. Ekrandaki sarı üçgenin yönünü donanımın gerçek açısına eşitle
                            map.pose.2 = payload.scan_angle; 

                            // 3. Veriyi haritaya işle (Robot şasesi sabit olduğu için heading 0.0)
                            map.process_ping(0.0, 0.0, 0.0, payload.scan_angle, payload.scan_dist);
                        }
                    } 
                } 
            }
        }
    });

    // NOT: THREAD 3 (Otonom ECU) araç sabit olduğu için bu test branch'inde kaldırılmıştır.

    // THREAD 1: UI Event Loop
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Stationary Radar Canvas",
        options,
        Box::new(move |_cc| Box::new(RoverDashboard { 
            grid: shared_grid, 
            selected_map: initial_map 
        })),
    )?;

    Ok(())
}