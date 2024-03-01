//! ## Mouse
//!
//! Definition of a mouse.

use std::collections::HashMap;

pub use winit::event::MouseButton;

use crate::event::event::Event;

/// Contains all the necessary data to interact with the mouse as an input element.
pub struct Mouse {
  buttons: HashMap<MouseButton, bool>,
}

impl Mouse {
  pub fn new() -> Self {
    let buttons = HashMap::new();
    Self {
      buttons,
    }
  }

  /// Set a button an its state.
  pub fn set_button(&mut self, button: MouseButton, is_pressed: bool) {
    self.buttons.insert(button, is_pressed);
  }

  /// Check the state of a button.
  pub fn is_pressed(&self, button: MouseButton) -> bool {
    self.buttons.get(&button).copied().unwrap_or(false)
  }

  /// Process the passed events.
  pub fn process_events(&mut self, event: Event) {
    match event {
      Event::MouseInput {
        button,
        is_pressed,
      } => {
        self.set_button(button, is_pressed);
      },
      _ => {},
    }
  }
}

impl Default for Mouse {
  fn default() -> Self {
    Self::new()
  }
}
