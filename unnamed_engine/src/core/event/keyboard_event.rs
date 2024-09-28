use strum::Display;
pub use winit::keyboard::PhysicalKey;
pub use winit::keyboard::KeyCode;

/// Events produced by the keyboard.
#[derive(Debug, PartialEq, Eq, Display)]
pub enum KeyboardEvent {
    /// The attached `PhysicalKey` was pressed.
    Press(PhysicalKey),
    /// The attached `PhysicalKey` was released.
    Release(PhysicalKey),
}
