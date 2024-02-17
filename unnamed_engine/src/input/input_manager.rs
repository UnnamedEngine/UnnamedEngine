//! ## Input Manager

use crate::event::event::Event;

use super::{keyboard::Keyboard, mouse::Mouse};

/// Responsible for handling all the possible inputs, if an input can be handled it must be here.
pub struct InputManager {
  pub keyboard: Keyboard,
  pub mouse: Mouse,
}

impl InputManager {
  pub fn new() -> Self {
    let keyboard = Keyboard::new();
    let mouse = Mouse::new();
    Self {
      keyboard,
      mouse,
    }
  }

  /// Handles the events and replicate then to other areas.
  pub fn process_events(&mut self, event: &Event) {
    self.keyboard.process_events(event.clone());
    self.mouse.process_events(event.clone());
  }
}
