use std::str;
use tokio::net::UdpSocket;

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

    let mut buf = [0u8; 1024];

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, src)) => {
                if let Ok(raw_str) = str::from_utf8(&buf[..len]) {
                    match parse_payload(raw_str) {
                        Some(payload) => {
                            println!(
                                "[{}] x: {}, y: {}, heading: {}, angle: {}, dist: {}", 
                                src, 
                                payload.base_x, 
                                payload.base_y, 
                                payload.heading, 
                                payload.scan_angle, 
                                payload.scan_dist
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
