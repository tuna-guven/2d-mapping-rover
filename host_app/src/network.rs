#[derive(Debug)]
pub struct SlamPayload {
    pub base_x: f32, 
    pub base_y: f32, 
    pub heading: f32, 
    pub scan_angle: f32,
    pub scan_dist: f32,
}

pub fn parse_payload(raw: &str) -> Option<SlamPayload> {
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