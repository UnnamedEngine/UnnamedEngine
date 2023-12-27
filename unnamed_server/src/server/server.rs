////////////////////////////////////////////////////////////////////////////////
//
//                ██╗   ██╗███╗   ██╗███████╗███╗   ██╗
//                ██║   ██║████╗  ██║██╔════╝████╗  ██║
//                ██║   ██║██╔██╗ ██║█████╗  ██╔██╗ ██║
//                ██║   ██║██║╚██╗██║██╔══╝  ██║╚██╗██║
//                ╚██████╔╝██║ ╚████║███████╗██║ ╚████║
//                 ╚═════╝ ╚═╝  ╚═══╝╚══════╝╚═╝  ╚═══╝ SERVER
//
////////////////////////////////////////////////////////////////////////////////
// ? Defines and implements the Server wrapper.
use unnamed_engine::{core::engine::*, event::{self, event::KeyCode}};

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
                    event::event::Event::Keyboard {
                        key,
                        is_pressed } => {
                            if is_pressed {
                                match key {
                                    // Engine should stop
                                    KeyCode::Escape => {
                                        engine.stop();
                                    }
                                    _ => {},
                                }
                            }
                        },
                }
            });
    }
}
