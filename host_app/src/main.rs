use eframe::egui;
use std::error::Error;
use std::str;
use std::sync::{Arc, Mutex};
use std::thread;
use std::io::{BufRead, BufReader};
use std::time::Duration;


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

    // THREAD 2: Gözler (Doğrudan USB Seri Portundan Okur)
    thread::spawn(move || {
        // Arkadaşının Linux portu. Sen Windows'ta test ederken burayı "COM9" yapmalısın.
        let port_name = "/dev/ttyUSB0"; 
        let baud_rate = 115200;

        let port_result = serialport::new(port_name, baud_rate)
            .timeout(Duration::from_millis(10))
            .open();

        match port_result {
            Ok(port) => {
                println!("✅ Seri port başarıyla açıldı: {}", port_name);
                let mut reader = BufReader::new(port);
                let mut line = String::new();
                
                loop {
                    line.clear();
                    // Satır satır (newline karakterine kadar) Arduino'dan oku
                    if let Ok(_) = reader.read_line(&mut line) {
                        let raw_str = line.trim();
                        
                        // Eski payload parser'a gönder
                        if let Some(payload) = parse_payload(raw_str) {
                            if let Ok(mut map) = grid_for_udp.lock() {
                                // 1. Radar masada sabit, konumu hep (0,0)
                                map.pose.0 = 0.0;
                                map.pose.1 = 0.0;
                                
                                // 2. Ekrandaki sarı üçgenin yönünü donanımın gerçek açısına eşitle
                                map.pose.2 = payload.scan_angle; 

                                // 3. Veriyi haritaya işle
                                map.process_ping(0.0, 0.0, 0.0, payload.scan_angle, payload.scan_dist);
                            }
                        } 
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Seri port açılamadı '{}'. Hata: {}", port_name, e);
                eprintln!("Linux kullanıyorsanız porta izin verdiğinizden emin olun: sudo chmod a+rw {}", port_name);
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