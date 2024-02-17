//! ## Event
//!
//! Defines the events that will be used internally and passed the applications.
pub use winit::keyboard::KeyCode;
pub use winit::event::MouseButton;

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
  MouseMoved {
    x: u32,
    y: u32,
  },
  MouseInput {
    button: MouseButton,
    is_pressed: bool,
  },
  MouseScroll {
    delta: (f32, f32),
  }
}
