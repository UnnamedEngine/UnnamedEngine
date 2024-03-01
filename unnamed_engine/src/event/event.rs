//! ## Event
//!
//! Defines the events that will be used internally and passed the applications.
pub use winit::event::MouseButton;
pub use winit::keyboard::KeyCode;

/// All the possible events for UnnamedEngine are registered here.
#[derive(Debug, Clone, Copy)]
pub enum Event {
  Shutdown,
  Resize {
    width: u32,
    height: u32,
  },
  KeyboardInput {
    key: KeyCode,
    is_pressed: bool,
  },
  MouseMotion {
    x: f32,
    y: f32,
  },
  MousePosition {
    x: u32,
    y: u32,
  },
  MouseInput {
    button: MouseButton,
    is_pressed: bool,
  },
  MouseScroll {
    delta: (f32, f32),
  },
  Redraw,
}
