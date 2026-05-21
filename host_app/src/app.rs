use crate::grid::{OccupancyGrid, CELL_SIZE};
use eframe::egui;
use std::sync::{Arc, Mutex};

pub struct RadarDashboard {
    pub grid: Arc<Mutex<OccupancyGrid>>,
}

impl eframe::App for RadarDashboard {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_millis(33)); // 30 FPS Cap

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("360° Stationary Topographical Radar");
            ui.separator();

            if let Ok(mut grid_lock) = self.grid.try_lock() {
                // --- DASHBOARD CONTROLS ---
                ui.horizontal(|ui| {
                    if ui.button("🧹 Clear Radar Sweep").clicked() {
                        grid_lock.cells.clear();
                        grid_lock.inflated_cells.clear();
                        grid_lock.frontiers.clear();
                    }

                    let mut obs_cells = 0;
                    for &val in grid_lock.cells.values() {
                        if val > 0.0 {
                            obs_cells += 1;
                        }
                    }
                    ui.label(
                        egui::RichText::new(format!(
                            " | 📡 Mapping Active ({} points detected)",
                            obs_cells
                        ))
                        .color(egui::Color32::GREEN),
                    );
                });
                ui.separator();

                // --- CANVAS ALLOCATION ---
                let (response, painter) =
                    ui.allocate_painter(ui.available_size(), egui::Sense::hover());
                let center = response.rect.center();
                let scale = 2.0;

                // Draw Explored Map
                for (&(grid_x, grid_y), &val) in &grid_lock.cells {
                    let color = if val > 0.0 {
                        egui::Color32::RED // Physical wall hit
                    } else if val < 0.0 {
                        egui::Color32::from_rgb(10, 30, 10) // Faint dark green for empty space
                    } else {
                        continue;
                    };

                    let x_pos = center.x + (grid_x as f32 * CELL_SIZE * scale);
                    let y_pos = center.y - (grid_y as f32 * CELL_SIZE * scale);
                    let rect = egui::Rect::from_center_size(
                        egui::pos2(x_pos, y_pos),
                        egui::vec2(CELL_SIZE * scale, CELL_SIZE * scale),
                    );
                    painter.rect_filled(rect, 0.0, color);
                }

                // Draw the Stationary Sensor (Center Point)
                painter.circle_filled(center, 5.0, egui::Color32::YELLOW);
            } else {
                ui.label("Waiting for Serial Data...");
            }
        });
    }
}
