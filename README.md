# Autonomous 2D SLAM Rover

A high-performance, fully autonomous 2D SLAM (Simultaneous Localization and Mapping) simulated rover built in Rust. This project implements a complete autonomous navigation stack from scratch, allowing a simulated agent to explore unknown environments, identify frontiers, and navigate complex geometries using a custom-built Engine Control Unit (ECU) and A* Pathfinding.

Designed with native thread isolation and strict event-loop management, this architecture guarantees stable UI performance.

## Key Features

* **Autonomous State Machine (ECU):** A dedicated background thread acts as the rover's brain, operating at 20Hz. It seamlessly transitions between `THINK` (A* path planning and frontier scoring) and `DRIVE` (kinematic translation) states.
* **Frontier-Based Exploration:** Mathematically identifies the "shadow line" between known free space and the unknown void. Frontiers are scored by distance and cluster size to prioritize meaningful exploration and prevent the rover from fixating on dead-end fragments.
* **Fallback Navigation Engine:** Automatically detects unreachable frontiers (e.g., blocked by the rover's inflation radius or tight corners) and re-routes to the next best target, preventing logic deadlocks.
* **Dynamic Environment Swapping:** Hot-swap between multiple test tracks (Hourglass, Rooms, Pillars) on the fly without recompiling.
* **Flood-Fill Safe Spawning:** Uses a Breadth-First Search (BFS) to map the interior of any loaded environment, guaranteeing the rover instantly teleports to a mathematically collision-free coordinate when reset.

## How It Works

To prevent lockups and UI deadlocks, the application completely bypasses heavy async runtimes (like Tokio) in favor of a strictly decoupled, native `std::thread` architecture:

1. **Thread 1: UI Event Loop (The Face)**
   * Built with `eframe`/`egui`.
   * Strictly capped at 30 FPS (`ctx.request_repaint_after`) to give the OS graphics pipeline time to breathe.
   * Holds the master `try_lock` for rendering. If the background threads are busy, it gracefully drops a frame rather than freezing the application.

2. **Thread 2: Sensor Input (The Eyes)**
   * A lightweight, headless UDP receiver bound to `0.0.0.0:4210`.
   * Processes incoming CSV telemetry payloads (`scan_angle`, `scan_dist`) from a Python fuzzer script.
   * Utilizes **Bresenham's Line Algorithm** for high-performance raycasting to simulate physical ultrasonic sensor beam physics and update the occupancy grid in real-time.

3. **Thread 3: Autonomous ECU (The Brain)**
   * Operates entirely on isolated physical CPU cores.
   * **State: THINK** - Clones the current map into RAM, runs O(N) array calculations to find the closest reachable frontier, and calculates an **A*** path around inflated obstacle boundaries.
   * **State: DRIVE** - Calculates the `atan2` heading to the next node and performs kinematic translation at a set speed, dynamically pulling the sensor cone through the fog of war.

## Modular Structure

The codebase is modularized for maintainability and scalability:

* `src/main.rs`: Thread orchestration, state synchronization (`Arc<Mutex<T>>`), and application entry point.
* `src/app.rs`: Immediate-mode UI dashboard, handling live rendering, geometry scaling, and map selection controls.
* `src/grid.rs`: The core mathematical engine containing the `OccupancyGrid`, A* implementation, Bresenham raycasting, and boundary inflation logic.
* `src/maps.rs`: The environment generation facility, housing various test tracks (Rooms, Hourglass, Pillars) used to stress-test the navigation logic.
* `src/network.rs`: UDP parsing layer that extracts payload data from the simulated sensor array.

## Getting Started

### Prerequisites
* Rust toolchain (`cargo`, `rustc`)
* Linux dependencies for `eframe` (if running on Wayland/X11)
* A telemetry fuzzer (e.g., `test_sender.py`) sending CSV packets to `0.0.0.0:4210`.

### Running the System
Because the A* algorithm and geometry tessellation are mathematically heavy, you **must** run this project in release mode. Compiler optimizations will drastically reduce pathfinding calculation times from seconds to sub-milliseconds.

```bash
cargo run --release
