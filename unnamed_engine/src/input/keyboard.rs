//! ## Keyboard
//!
//! Definition of a keyboard.

use std::collections::HashMap;
pub use winit::keyboard::KeyCode;

use crate::event::event::Event;

/// Contains all the necessary data to interact with the keyboard as an input element.
pub struct Keyboard {
  keys: HashMap<KeyCode, bool>,
}

impl Keyboard {
  pub fn new() -> Self {
    let keys = HashMap::new();
    Self { keys }
  }

  /// Set a key and its state.
  pub fn set_key(&mut self, keycode: KeyCode, is_pressed: bool) {
    self.keys.insert(keycode, is_pressed);
  }

  /// Check the state of a key.
  pub fn is_pressed(&self, keycode: KeyCode) -> bool {
    self.keys.get(&keycode).copied().unwrap_or(false)
  }

  /// Preparation before events get passed.
  pub fn prepare(&mut self) {}

  /// Process the passed events
  pub fn process_events(&mut self, event: Event) {
    match event {
      Event::KeyboardInput { key, is_pressed } => {
        self.set_key(key, is_pressed);
      }
      _ => {}
    }
  }
}

impl Default for Keyboard {
  fn default() -> Self {
    Self::new()
  }
}
