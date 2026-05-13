use std::net::UdpSocket;
use std::str;

struct SlamPayload {
    base_x: f32, // The position of the rover in the world coordinate system x
    base_y: f32, // The position of the rover in the world coordinate system y
    heading: f32, // The orientation of the rover in the world coordinate system
    scan_angle: f32, // The angle of the laser scan relative to the rover
    scan_dist: f32, // The distance of the laser scan
}


fn parse(raw: &str) -> Option<SlamPayload> {    // parsers the raw string slam payload into a SlamPayload struct
    let parts: Vec<&str> = raw.trim().split(',').collect();

    if parts.len() != 5 {
        return None;
    }

    Some(SlamPayload {
        base_x:     parts[0].trim().parse().ok()?,
        base_y:     parts[1].trim().parse().ok()?,
        heading:    parts[2].trim().parse().ok()?,
        scan_angle: parts[3].trim().parse().ok()?,
        scan_dist:  parts[4].trim().parse().ok()?,
    })
}

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:5000")
        .expect("Failed to bind UDP port");

    println!("Listening on port 5000...");

    let mut buf = [0u8; 1024];

    loop {
        match socket.recv_from(&mut buf) {
            Ok((len, src)) => {
                match str::from_utf8(&buf[..len]) {
                    Ok(raw) => {
                        match parse(raw) {
                            Some(payload) => println!(
                                "[{}] x={} y={} heading={} angle={} dist={}",
                                src,
                                payload.base_x, payload.base_y,
                                payload.heading, payload.scan_angle,
                                payload.scan_dist
                            ),
                            None => eprintln!("Bad packet: {:?}", raw.trim()),
                        }
                    }
                    Err(_) => eprintln!("Non-UTF8 packet from {}", src),
                }
            }
            Err(e) => eprintln!("Socket error: {}", e),
        }
    }
}