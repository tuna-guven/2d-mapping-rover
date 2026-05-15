# Autonomous 2D SLAM & Navigation

A high-performance, multi-threaded 2D SLAM (Simultaneous Localization and Mapping) rover built in Rust. This project implements a full autonomous navigation stack, allowing a simulated rover to explore unknown environments, identify frontiers, and navigate complex geometries using a custom-built State Machine and A* Pathfinding.

## Key Architectural Features

* **Frontier-Based Exploration:** Mathematically identifies the "shadow line" between known free space and the unknown void. Frontiers are scored by distance and cluster size to prioritize meaningful exploration.
* **3-Thread Isolated Architecture:**
    1.  **Main/UI Thread:** Runs the `egui` event loop at a stable 30 FPS, optimized for Linux Wayland/GNOME stability.
    2.  **Sensor Thread:** A dedicated `std::net::UdpSocket` loop that processes asynchronous telemetry packets without blocking the UI.
    3.  **ECU/Planner Thread:** The "Brain" of the rover. Operates at 20Hz, managing the State Machine and heavy A* math on a background CPU core.
* **Autonomous State Machine:**
    * **THINK:** Snapshots the current map to plan an optimal route to the best reachable frontier.
    * **DRIVE:** Handles kinematic translation and rotation, moving the rover along the planned path.
* **Fallback Navigation Engine:** Automatically detects unreachable frontiers (blocked by inflation or tight corners) and re-routes to the next best target, preventing logic deadlocks.
* **Secure Environment:** Developed and tested on linux, utilizing native thread isolation to remain performant even with SMT disabled.

## Technical Stack

* **Language:** Rust (Systems-level performance and memory safety).
* **GUI:** `eframe` / `egui` (Immediate-mode, non-blocking rendering).
* **Pathfinding:** Custom A* implementation with an iteration-depth failsafe.
* **Raycasting:** Bresenham’s Line Algorithm for simulated ultrasonic sensor physics.
* **Math:** Pattern-matching for coordinate unwrapping and `atan2` for heading calculations.

## Modular Structure

* `src/network.rs`: UDP packet parsing and telemetry extraction.
* `src/grid.rs`: Occupancy Grid logic, raycasting, and pathfinding.
* `src/maps.rs`: Environment generation (Hourglass, Rooms, Pillars).
* `src/app.rs`: UI Dashboard, rendering logic, and map selection controls.
* `src/main.rs`: Thread orchestration and application entry point.

## Getting Started

### Prerequisites
* Rust toolchain (`cargo`, `rustc`)
* A telemetry fuzzer (e.g., `test_sender.py`) sending CSV packets to `0.0.0.0:4210`.

### Running the System
For optimal performance and to avoid UI stutters during complex path calculations, always run in release mode:

```bash
cargo run --release