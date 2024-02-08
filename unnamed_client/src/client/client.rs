////////////////////////////////////////////////////////////////////////////////
//                ██╗   ██╗███╗   ██╗███████╗███╗   ██╗                       //
//                ██║   ██║████╗  ██║██╔════╝████╗  ██║                       //
//                ██║   ██║██╔██╗ ██║█████╗  ██╔██╗ ██║                       //
//                ██║   ██║██║╚██╗██║██╔══╝  ██║╚██╗██║                       //
//                ╚██████╔╝██║ ╚████║███████╗██║ ╚████║                       //
//                 ╚═════╝ ╚═╝  ╚═══╝╚══════╝╚═╝  ╚═══╝ CLIENT                //
////////////////////////////////////////////////////////////////////////////////
// ? Defines and implements the basic Client wrapper.
use unnamed_engine::{core::engine::*, event::{self, event::KeyCode}};

pub struct Client {
  engine: Engine,
}

impl Client {
  pub fn new(title: String) -> Self {
    Client {
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
