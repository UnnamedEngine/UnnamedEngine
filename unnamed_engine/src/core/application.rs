//! ## Application
//!
//! Responsible for providing a way to create applications on top of the engine.
use crate::event::event::Event;

pub struct ApplicationDescriptor {
  pub title: String,
}

pub trait Application {
  fn new(desc: ApplicationDescriptor) -> Self;
  fn start(&mut self);
  fn update(&mut self);
  fn render(&mut self);
  fn render_gui(&mut self);
  fn on_event(&mut self, event: Event);
}
