pub mod worker;

/// Helper that defines a `FnOnce` that will be sent to the `ThreadPool` and
/// executed by a `Worker`.
pub type Job = Box<dyn FnOnce() + Send + 'static>;
