use eframe::egui;
use std::sync::{Arc, Mutex};
use crate::grid::{OccupancyGrid, CELL_SIZE};
use crate::maps::{generate_map, MapType}; // Added MapType

pub struct RoverDashboard {
    pub grid: Arc<Mutex<OccupancyGrid>>,
    pub selected_map: MapType, // Track current selection
}

impl eframe::App for RoverDashboard {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_millis(33)); 

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("2D Mapping Rover - Fully Autonomous SLAM");
            ui.separator();

            if let Ok(mut grid_lock) = self.grid.try_lock() {
                
                // --- 1. DASHBOARD CONTROLS ---
                ui.horizontal(|ui| {
                    // Map Selector Dropdown
                    let prev_map = self.selected_map;
                    egui::ComboBox::from_label("Active Environment")
                        .selected_text(format!("{:?}", self.selected_map))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.selected_map, MapType::Hourglass, "Hourglass");
                            ui.selectable_value(&mut self.selected_map, MapType::Rooms, "Rooms");
                            ui.selectable_value(&mut self.selected_map, MapType::Pillars, "Pillars");
                        });

                    // If user changed the map, trigger the hard reset
                    if prev_map != self.selected_map {
                        let new_gt = generate_map(self.selected_map);
                        grid_lock.reset_with_map(new_gt);
                    }

                    ui.add_space(20.0);
                    if ui.button("🎲 Random Respawn").clicked() {
                        grid_lock.cells.clear();
                        grid_lock.current_path.clear();
                        grid_lock.frontiers.clear();
                        grid_lock.inflated_cells.clear();
                        grid_lock.teleport_random();
                    }
                });

                // Metrics
                let mut free_cells = 0;
                let mut obs_cells = 0;
                for &val in grid_lock.cells.values() {
                    if val < 0.0 { free_cells += 1; }
                    if val > 0.0 { obs_cells += 1; }
                }
                
                let area_m2 = (free_cells as f32) * 0.0025; 
                let is_done = grid_lock.frontiers.is_empty() && free_cells > 20;

                ui.horizontal(|ui| {
                    ui.label(format!("Free Area: {:.4} m²", area_m2));
                    ui.label(format!(" | Obstacles: {}", obs_cells));

                    if is_done {
                        ui.label(egui::RichText::new(" | 🎉 MAPPING COMPLETE").color(egui::Color32::GREEN).strong());
                    } else if free_cells > 0 {
                        ui.label(egui::RichText::new(" | 📡 Exploring...").color(egui::Color32::YELLOW));
                    }
                });
                ui.separator();
                // -----------------------------------

                let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::hover());
                let center = response.rect.center(); 
                let scale = 2.0; 

                for &(grid_x, grid_y) in &grid_lock.ground_truth {
                    if !grid_lock.cells.contains_key(&(grid_x, grid_y)) {
                        let x_pos = center.x + (grid_x as f32 * CELL_SIZE * scale);
                        let y_pos = center.y - (grid_y as f32 * CELL_SIZE * scale); 
                        let rect = egui::Rect::from_center_size(
                            egui::pos2(x_pos, y_pos),
                            egui::vec2(CELL_SIZE * scale, CELL_SIZE * scale),
                        );
                        painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(50, 100, 255));
                    }
                }

                for (&(grid_x, grid_y), &val) in &grid_lock.cells {
                    let color = if val > 0.0 {
                        egui::Color32::RED
                    } else if grid_lock.frontiers.contains(&(grid_x, grid_y)) {
                        egui::Color32::YELLOW 
                    } else if grid_lock.inflated_cells.contains(&(grid_x, grid_y)) {
                        egui::Color32::from_rgb(139, 0, 0)
                    } else if val < 0.0 {
                        egui::Color32::GREEN 
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

                for &(px, py) in &grid_lock.current_path {
                    let x_pos = center.x + (px as f32 * CELL_SIZE * scale);
                    let y_pos = center.y - (py as f32 * CELL_SIZE * scale);
                    let rect = egui::Rect::from_center_size(
                        egui::pos2(x_pos, y_pos),
                        egui::vec2(CELL_SIZE * scale, CELL_SIZE * scale),
                    );
                    painter.rect_filled(rect, 0.0, egui::Color32::LIGHT_BLUE);
                }

                let robot_x = center.x + (grid_lock.pose.0 * scale);
                let robot_y = center.y - (grid_lock.pose.1 * scale);
                let heading_rad = grid_lock.pose.2.to_radians();

                let tip_x = robot_x + (7.5 * scale * heading_rad.cos());
                let tip_y = robot_y - (7.5 * scale * heading_rad.sin());
                let left_x = robot_x + (7.5 * scale * (heading_rad + 2.5).cos());
                let left_y = robot_y - (7.5 * scale * (heading_rad + 2.5).sin());
                let right_x = robot_x + (7.5 * scale * (heading_rad - 2.5).cos());
                let right_y = robot_y - (7.5 * scale * (heading_rad - 2.5).sin());

                painter.add(egui::Shape::convex_polygon(
                    vec![egui::pos2(tip_x, tip_y), egui::pos2(left_x, left_y), egui::pos2(right_x, right_y)],
                    egui::Color32::YELLOW,
                    egui::Stroke::new(1.0, egui::Color32::WHITE),
                ));

            } else {
                ui.label("Map Synchronizing...");
            }
        });
    }
}