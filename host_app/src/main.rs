uuse eframe::egui;
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
    // Sabit radar testi için boş bir harita şablonu başlatıyoruz
    let initial_map = MapType::Rooms; 
    let ground_truth = generate_map(initial_map); 

    // Ortak hafıza (Grid) başlangıcı
    let shared_grid = Arc::new(Mutex::new(OccupancyGrid::new(ground_truth)));
    let grid_for_udp = shared_grid.clone();

    // THREAD 2: Fiziksel Donanımdan (Python Köprüsünden) Veri Alan Kısım
    thread::spawn(move || {
        let socket = UdpSocket::bind("0.0.0.0:4210").expect("Failed to bind UDP");
        let mut buf = [0u8; 1024];
        
        loop {
            if let Ok((len, _)) = socket.recv_from(&mut buf) {
                if let Ok(raw_str) = str::from_utf8(&buf[..len]) {
                    if let Some(payload) = parse_payload(raw_str) {
                        if let Ok(mut map) = grid_for_udp.lock() {
                            // 1. Robot sabit olduğu için konumu hep merkezde (0.0, 0.0) tut
                            map.pose.0 = 0.0;
                            map.pose.1 = 0.0;
                            
                            // 2. Ekrandaki sarı üçgenin yönünü donanımdaki anlık radar açısına eşitle
                            map.pose.2 = payload.scan_angle; 

                            // 3. Gerçek mesafeyi haritaya işle
                            map.process_ping(0.0, 0.0, 0.0, payload.scan_angle, payload.scan_dist);
                        }
                    } 
                } 
            }
        }
    });

    // NOTE: Eski THREAD 3 (ECU / Otonom Sürüş Motoru) araç fiziksel olarak 
    // hareket etmeyeceği için mimariden tamamen temizlendi.

    // THREAD 1: Arayüz Döngüsü (UI)
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Stationary 2D Radar Canvas",
        options,
        Box::new(move |_cc| Box::new(RoverDashboard { 
            grid: shared_grid, 
            selected_map: initial_map 
        })),
    )?;

    Ok(())
}