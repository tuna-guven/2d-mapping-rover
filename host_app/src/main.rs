use std::str;
use tokio::net::UdpSocket;

// grid modülünü içeri aktarıyoruz
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

    // Reject the packet if there is trailing garbage data (more than 5 parts)
    if parts.next().is_some() {
        return None;
    }

    Some(payload)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let socket = match UdpSocket::bind("0.0.0.0:4210").await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to bind to UDP port: {}", e);
            return Err(e);
        }
    };

    println!("✅ Listening for high-speed rover telemetry on UDP port 4210...");

    // Haritamızı (Grid) döngüden önce başlatıyoruz
    let mut map = OccupancyGrid::new();
    let mut buf = [0u8; 1024];

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, src)) => {
                if let Ok(raw_str) = str::from_utf8(&buf[..len]) {
                    match parse_payload(raw_str) {
                        Some(payload) => {
                            // 1. Sensörün mutlak açısını bul (Aracın yönü + Sensörün yönü) ve radyana çevir
                            let global_angle_rad = (payload.heading + payload.scan_angle).to_radians();
                            
                            // 2. Trigonometri ile çarpma noktasının Global X ve Y koordinatlarını hesapla
                            let ping_global_x = payload.base_x + (payload.scan_dist * global_angle_rad.cos());
                            let ping_global_y = payload.base_y + (payload.scan_dist * global_angle_rad.sin());

                            // 3. Haritayı güncelliyoruz (Ray-casting)
                            map.process_ping(payload.base_x, payload.base_y, ping_global_x, ping_global_y);

                            println!(
                                "[{}] Paket işlendi! Grid'deki bilinen hücre sayısı: {}", 
                                src, 
                                map.cells.len()
                            );
                        }
                        None => eprintln!("⚠️ Malformed payload from {}: '{}'", src, raw_str.trim()),
                    }
                } else {
                    eprintln!("⚠️ Received non-UTF8 packet from {}", src);
                }
            }
            Err(e) => eprintln!("🛑 Socket receive error: {}", e),
        }
    }
}