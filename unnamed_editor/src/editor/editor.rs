//! ## Editor
//!
//! Defines and implements the editor application.
use unnamed_engine::{core::engine::*, event::{self, event::KeyCode}};

/// ## Editor
///
/// This is an editor application that contains the entire engine alongside the editor tools.
pub struct Editor {
  engine: Engine,
}

impl Editor {
  pub fn new(title: String) -> Self {
    Editor {
      engine: Engine::new(title)
    }
  }

  pub fn start(&mut self) {
    self.engine.start(
      |engine| {

      },
      |engine| {

      },
      |engine| {

      },
      |engine, event| {
        // Handle events
        match event {
          // Keyboard event
          event::event::Event::KeyboardInput {
            key,
            is_pressed } => {
              if is_pressed {
                match key {
                  // Engine should stop
                  KeyCode::Escape => {
                    engine.stop();
                  }
                  _ => {},
                }
              }
            },
          _ => {}
        }
      });
  }
}
