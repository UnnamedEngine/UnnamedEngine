//! ## Server
//!
//! Defines and implements the server application.
use unnamed_engine::{core::engine::*, event::{self, event::KeyCode}};

/// ## Server
///
/// This is a server application that contains the server and some tooling, its intended to be
/// executed for dedicated servers.
pub struct Server {
    engine: Engine,
}

impl Server {
    pub fn new(title: String) -> Self {
        Server {
            engine: Engine::new(title)
        }
    }

    pub fn start(&mut self) {
        self.engine.start(
            |engine| {

            },
            |engine| {

            },
            |engine| {

            },
            |engine, event| {
                // Handle events
                match event {
                    // Keyboard event
                    event::event::Event::KeyboardInput {
                        key,
                        is_pressed } => {
                            if *is_pressed {
                                match key {
                                    // Engine should stop
                                    KeyCode::Escape => {
                                        engine.stop();
                                    }
                                    _ => {},
                                }
                            }
                        },
                    _ => {}
                }
            });
    }
}
