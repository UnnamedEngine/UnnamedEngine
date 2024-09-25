use strum::Display;

/// Events produced by the `Engine`.
#[derive(Debug, PartialEq, Eq, Display)]
pub enum EngineEvent {
    /// The `Engine` just got started.
    Started,
    /// The `Engine` is preparing for shutdown.
    Shutdown,
    /// The `Engine` has stopped.
    Stopped,
}
