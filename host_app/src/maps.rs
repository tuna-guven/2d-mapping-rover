use std::collections::HashSet;
use crate::grid::{OccupancyGrid, bresenham_line};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapType {
    Hourglass,
    Rooms,
    Pillars,
}

pub fn generate_map(map_type: MapType) -> HashSet<(i32, i32)> {
    match map_type {
        MapType::Hourglass => generate_hourglass(),
        MapType::Rooms => generate_rooms(),
        MapType::Pillars => generate_pillars(),
    }
}

fn add_wall(gt: &mut HashSet<(i32, i32)>, x0: f32, y0: f32, x1: f32, y1: f32) {
    let p0 = OccupancyGrid::world_to_grid(x0, y0);
    let p1 = OccupancyGrid::world_to_grid(x1, y1);
    for p in bresenham_line(p0.0, p0.1, p1.0, p1.1) {
        gt.insert(p);
    }
}

fn generate_hourglass() -> HashSet<(i32, i32)> {
    let mut gt = HashSet::new();
    add_wall(&mut gt, -80.0, 100.0, 80.0, 100.0);
    add_wall(&mut gt, -80.0, -100.0, 80.0, -100.0);
    add_wall(&mut gt, -80.0, 100.0, -80.0, 50.0);
    add_wall(&mut gt, -80.0, 50.0, -20.0, 0.0);
    add_wall(&mut gt, -20.0, 0.0, -80.0, -50.0);
    add_wall(&mut gt, -80.0, -50.0, -80.0, -100.0);
    add_wall(&mut gt, 80.0, 100.0, 80.0, 50.0);
    add_wall(&mut gt, 80.0, 50.0, 20.0, 0.0);
    add_wall(&mut gt, 20.0, 0.0, 80.0, -50.0);
    add_wall(&mut gt, 80.0, -50.0, 80.0, -100.0);
    gt
}

fn generate_rooms() -> HashSet<(i32, i32)> {
    let mut gt = HashSet::new();
    // Outer Walls
    add_wall(&mut gt, -100.0, 100.0, 100.0, 100.0);
    add_wall(&mut gt, -100.0, -100.0, 100.0, -100.0);
    add_wall(&mut gt, -100.0, 100.0, -100.0, -100.0);
    add_wall(&mut gt, 100.0, 100.0, 100.0, -100.0);

    // Divider wall with a narrow doorway in the middle
    add_wall(&mut gt, 0.0, 100.0, 0.0, 25.0);
    add_wall(&mut gt, 0.0, -25.0, 0.0, -100.0);
    gt
}

fn generate_pillars() -> HashSet<(i32, i32)> {
    let mut gt = HashSet::new();
    // Outer Walls
    add_wall(&mut gt, -100.0, 100.0, 100.0, 100.0);
    add_wall(&mut gt, -100.0, -100.0, 100.0, -100.0);
    add_wall(&mut gt, -100.0, 100.0, -100.0, -100.0);
    add_wall(&mut gt, 100.0, 100.0, 100.0, -100.0);

    // Four square pillars for the rover to navigate around
    add_wall(&mut gt, -60.0, 60.0, -40.0, 60.0); add_wall(&mut gt, -60.0, 40.0, -40.0, 40.0);
    add_wall(&mut gt, -60.0, 60.0, -60.0, 40.0); add_wall(&mut gt, -40.0, 60.0, -40.0, 40.0);

    add_wall(&mut gt, 40.0, 60.0, 60.0, 60.0); add_wall(&mut gt, 40.0, 40.0, 60.0, 40.0);
    add_wall(&mut gt, 40.0, 60.0, 40.0, 40.0); add_wall(&mut gt, 60.0, 60.0, 60.0, 40.0);

    add_wall(&mut gt, -60.0, -40.0, -40.0, -40.0); add_wall(&mut gt, -60.0, -60.0, -40.0, -60.0);
    add_wall(&mut gt, -60.0, -40.0, -60.0, -60.0); add_wall(&mut gt, -40.0, -40.0, -40.0, -60.0);

    add_wall(&mut gt, 40.0, -40.0, 60.0, -40.0); add_wall(&mut gt, 40.0, -60.0, 60.0, -60.0);
    add_wall(&mut gt, 40.0, -40.0, 40.0, -60.0); add_wall(&mut gt, 60.0, -40.0, 60.0, -60.0);
    gt
}