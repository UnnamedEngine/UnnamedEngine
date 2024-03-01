//! ## Module
//!
//!
//! Defines the module trait.
use std::error::Error;

use instant::Duration;
use winit::event::WindowEvent;

use crate::{event::event::Event, renderer::viewport::Viewport};

/// UnnamedEngine is built by modules, each modules is a individual unit that can interact with
/// other modules.
pub trait Module {
  fn process_events(&mut self, event: Event) -> Result<(), Box<dyn Error>>;
  fn update(&mut self, dt: Duration) -> Result<(), Box<dyn Error>>;
  fn render(&mut self) -> Result<(), Box<dyn Error>> {
    Ok(())
  }
  fn viewport(&mut self) -> Option<&mut Viewport> {
    None
  }
  fn process_window_events(&mut self, _event: &WindowEvent) {}
}
