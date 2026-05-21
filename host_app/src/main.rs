use eframe::egui;
use std::collections::HashSet;
use std::error::Error;
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};
use std::thread;
<<<<<<< HEAD
use std::time::Duration;

=======
use std::io::{BufRead, BufReader};
use std::time::Duration;


mod network;
mod maps;
mod grid;
>>>>>>> refs/remotes/origin/feature/stationary-radar
mod app;
mod grid;

use app::RadarDashboard;
use grid::OccupancyGrid;

fn main() -> Result<(), Box<dyn Error>> {
<<<<<<< HEAD
    // Initialize a blank map with NO ground truth walls
    let shared_grid = Arc::new(Mutex::new(OccupancyGrid::new(HashSet::new())));
    let grid_for_serial = shared_grid.clone();

    // THREAD 2: High-Speed Serial Listener
    thread::spawn(move || {
        // Change this if your Nano mounts differently on secureblue
        let port_name = "/dev/ttyUSB0";
        let baud_rate = 38400;

        let port_builder =
            serialport::new(port_name, baud_rate).timeout(Duration::from_millis(100));

        match port_builder.open() {
            Ok(port) => {
                println!(
                    "📡 Serial port {} opened successfully at {} baud.",
                    port_name, baud_rate
                );
                let mut reader = BufReader::new(port);
                let mut line = String::new();

                loop {
                    line.clear();
                    if reader.read_line(&mut line).is_ok() {
                        let data = line.trim();
                        if data.is_empty() {
                            continue;
                        }

                        // Expecting exactly: "Angle,Distance" from the Nano (e.g., "-45.5,120.0")
                        let parts: Vec<&str> = data.split(',').collect();
                        if parts.len() == 2 {
                            // FIX: Added and to index into the vector before parsing
                            if let (Ok(scan_angle), Ok(scan_dist)) =
                                (parts[0].parse::<f32>(), parts[1].parse::<f32>())
                            {
                                if let Ok(mut map) = grid_for_serial.lock() {
                                    // Base is permanently locked to 0.0, 0.0, heading 0.0
                                    map.process_ping(0.0, 0.0, 0.0, scan_angle, scan_dist);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => eprintln!("❌ Failed to open serial port {}: {}", port_name, e),
        }
    });

=======
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

>>>>>>> refs/remotes/origin/feature/stationary-radar
    // THREAD 1: UI Event Loop
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
<<<<<<< HEAD
        "Stationary Radar",
        options,
        Box::new(move |_cc| Box::new(RadarDashboard { grid: shared_grid })),
=======
        "Stationary Radar Canvas",
        options,
        Box::new(move |_cc| Box::new(RoverDashboard { 
            grid: shared_grid, 
            selected_map: initial_map 
        })),
>>>>>>> refs/remotes/origin/feature/stationary-radar
    )?;

    Ok(())
}
