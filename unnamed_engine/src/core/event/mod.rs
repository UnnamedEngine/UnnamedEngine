use std::sync::mpsc::{self, TryRecvError};

use engine_event::EngineEvent;
use strum::Display;
use window_event::WindowEvent;

pub mod engine_event;
pub mod window_event;

/// Main enum that defines all our events.
///
/// There **is** a naming convention for any `Event`, past-sentence names are
/// refered to events that already occurred, other events are yet to occurr and
/// reacting to them can have some sort of influence.
///
/// **Events must contain only simple data.**
#[derive(Debug, PartialEq, Eq, Display)]
pub enum Event {
    /// Events produced by the `Engine`.
    Engine(EngineEvent),

    /// Events produced by a `winit::window::Window`.
    Window(WindowEvent),

    /// Only used during tests.
    #[cfg(test)]
    Dummy,
}

/// Creates a new `Dispatcher` and `Consumer` that are linked together.
///
/// Only one `Consumer` can exist, while multiple instances of a `Dispatcher`
/// can be used by calling `dispatcher.clone()`.
pub fn create_handler() -> (Dispatcher, Consumer) {
    let (sender, receiver) = mpsc::channel();

    let dispatcher = Dispatcher::new(sender);
    let consumer = Consumer::new(receiver);

    (dispatcher, consumer)
}

/// Helper that dispatches events.
pub struct Dispatcher {
    sender: mpsc::Sender<Event>,
}

impl Dispatcher {
    fn new(sender: mpsc::Sender<Event>) -> Self {
        Self {
            sender,
        }
    }

    /// Sends the `Event` to be processed where the `Consumer` is.
    pub fn send(&self, event: Event) {
        let _ = self.sender.send(event);
    }
}

impl Clone for Dispatcher {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

/// Helper that consumes events.
pub struct Consumer {
    receiver: mpsc::Receiver<Event>,
}

impl Consumer {
    fn new(receiver: mpsc::Receiver<Event>) -> Self {
        Self {
            receiver,
        }
    }

    /// Tries to receive an event. If no events are found it returns `None`.
    pub fn poll(&self) -> Option<Event> {
        match self.receiver.try_recv() {
            Ok(event) => Some(event),
            Err(err) => {
                match err {
                    TryRecvError::Empty => None,
                    _ => {
                        log::error!("Failed to receive event: {}", err.to_string());
                        None
                    }
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;

    #[test]
    fn create_dispatcher_consumer() {
        let (dispatcher, consumer) = create_handler();

        dispatcher.send(Event::Dummy);
        assert!(
            consumer.poll().is_some_and(|e| e == Event::Dummy),
            "Event should be able to be dispatched and consumed",
        );
    }

    #[test]
    fn event_being_consumed() {
        let (dispatcher, consumer) = create_handler();

        dispatcher.send(Event::Dummy);
        let _ = consumer.poll();
        assert!(
            consumer.poll().is_none(),
            "Event should be consumed at the first .poll()",
        );
    }

    #[test]
    fn multithreaded_consume() {
        let (dispatcher, consumer) = create_handler();

        let dispatcher_thread = thread::spawn(move || {
            dispatcher.send(Event::Dummy);
        });

        let consumer_thread = thread::spawn(move || {
            assert!(
                consumer.poll().is_some_and(|e| e == Event::Dummy),
                "Event should be able to be dispatched from one thread and consume from another",
            );
        });

        let _ = dispatcher_thread.join();
        let _ = consumer_thread.join();
    }
}
