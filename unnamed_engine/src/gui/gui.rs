use egui::Context;

use crate::renderer::middleware_renderer::RenderingStats;

pub fn gui(ui: &Context, rendering_stats: &RenderingStats) {
  egui::Window::new("Rendering Stats")
    .default_open(true)
    .max_width(1000.0)
    .max_height(800.00)
    .min_width(800.0)
    .min_height(800.0)
    .default_width(800.0)
    .default_height(800.0)
    .resizable(true)
    .movable(true)
    .show(&ui, |ui| {
      ui.label(format!("VRAM: {} KB", rendering_stats.bytes / 1024));
      ui.label(format!("Rendering Loop: {} ms", rendering_stats.delta.as_millis()));
    });
}
