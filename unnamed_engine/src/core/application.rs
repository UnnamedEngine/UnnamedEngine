//! ## Application
//!
//! UnnamedEngine uses applications to represent, well, applications, that way
//! the `Editor`, `Client` and `Server` are all applications.
//!
//! Generally speaking, we should have only one application being executed.

/// The public definer that is used to represent in which state we want a to add
/// a system to.
pub enum ApplicationState {
  Start,
  Update,
  Shutdown,
}

/// The current state of the application.
enum InternalApplicationState {
  Start,
  Update,
  Shutdown,
  Error,
}

pub struct Application {
}

