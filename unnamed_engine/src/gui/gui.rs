use egui::Context;

pub fn gui(ui: &Context) {
  egui::Window::new("Test")
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
      if ui.add(egui::Button::new("Click me!")).clicked() {
        log::info!("YOU CLICKED ME YAY!!!");
      }

      ui.label("Slider");
      ui.add(egui::Slider::new(&mut 0, 0..=100).text("number"));
      ui.end_row();

    });
}
