use strum::Display;
pub use winit::event::MouseButton;

/// Events produced by the keyboard.
#[derive(Debug, PartialEq, Eq, Display)]
pub enum MouseEvent {
    /// The attached `MouseButton` was pressed.
    ButtonPress(MouseButton),
    /// The attached `MouseButton` was released.
    ButtonRelease(MouseButton),
    /// The mouse was moved.
    Moved(u32, u32),
}
