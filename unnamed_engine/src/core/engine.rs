use strum::Display;

use super::event;

/// All the possible states a `Engine` can be at.
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
pub enum EngineState {
    /// `Engine` is currently stopped and can only be started with
    /// `Engine::start()`.
    Stopped,
    /// `Engine` is currently starting and will change into
    /// `EngineState::Running` once the starting process ends.
    Starting,
    /// `Engine` is currently running and can only change into
    /// `EngineState::Stopping`.
    Running,
    /// `Engine` is currently stopping and will change into
    /// `EngineState::Stopped` once the stopping process ends.
    Stopping,
}

/// Contains the major data required to run the application.
pub struct EngineData {
    /// Flags the current state of the `Engine`. This value will be read a lot
    /// and rarely will change.
    pub state: EngineState,
    /// `Event` dispatcher that can be freely cloned everywhere.
    pub event_dispatcher: event::Dispatcher,
}

pub struct Engine {
    /// Shared data.
    data: EngineData,
    /// Event consumer that will continuosly poll for events.
    event_consumer: event::Consumer,
}

impl Default for Engine {
    fn default() -> Self {
        // The logger is started here to make sure we have logging always
        // available

        // We do not want env_logger during tests
        #[cfg(not(test))]
        {
            // Read the env values that configure the logger
            let env = env_logger::Env::default()
                .filter_or("MY_LOG_LEVEL", "info")
                .write_style_or("MY_LOG_STYLE", "always");

            // Initialize the logger from the values
            // Now we can use `log::` everywhere without worrying
            env_logger::init_from_env(env);
        }

        let (event_dispatcher, event_consumer) = event::create_handler();

        let data = EngineData {
            state: EngineState::Stopped,
            event_dispatcher,
        };

        Self {
            data,
            event_consumer,
        }
    }
}

impl Engine {
    /// Start the `Engine`.
    pub fn run(&mut self) {
        match self.data.state {
            EngineState::Stopped => {
                self.data.state = EngineState::Starting;
                self.start();
            },
            _ => {
                log::error!(
                    "Can only start engine at '{}' state: tried to start at '{}' state",
                    EngineState::Stopped,
                    self.data.state,
                );
            }
        }
    }

    /// Stop the `Engine`.
    pub fn shutdown(&mut self) {
        match self.data.state {
            EngineState::Running => {
                self.data.state = EngineState::Stopping;
                self.data.event_dispatcher.send(
                    event::Event::Engine(event::engine_event::EngineEvent::Shutdown)
                );
                self.stop();
            },
            _ => {
                log::error!(
                    "Can only stop engine at '{}' state: tried to stop at '{}' state",
                    EngineState::Running,
                    self.data.state,
                );
            }
        }
    }

    /// Internal function that handles the `Engine` starting. awdkja kwjdkaj
    fn start(&mut self) {
        // TODO: there should be something here to start the engine
        self.data.state = EngineState::Running;
        self.data.event_dispatcher.send(
            event::Event::Engine(event::engine_event::EngineEvent::Started)
        );
        self.handle_all_events();
    }

    /// Internal function that handles the `Engine` stopping.
    fn stop(&mut self) {
        // TODO: there should be something here to stop the engine
        self.data.state = EngineState::Stopped;
        self.data.event_dispatcher.send(
            event::Event::Engine(event::engine_event::EngineEvent::Stopped)
        );
        self.handle_all_events();
    }

    /// Gets the current `EngineState`.
    pub fn state(&self) -> EngineState {
        self.data.state
    }

    /// Handle all pending events.
    fn handle_all_events(&self) {
        let mut pending = true;
        while pending {
            pending = self.handle_event();
        }
    }

    /// Handle a single event.
    fn handle_event(&self) -> bool {
        if let Some(event) = self.event_consumer.poll() {
            match event {
                event::Event::Engine(engine_event) => {
                    match engine_event {
                        event::engine_event::EngineEvent::Started => {
                            log::info!("Successfully started engine!");
                        },
                        event::engine_event::EngineEvent::Shutdown => {
                            log::info!("Engine preparing for graceful shutdown!");
                        },
                        event::engine_event::EngineEvent::Stopped => {
                            log::info!("Engine gracefully stopped!");
                            log::info!("See you again :D");
                        },
                    }
                },

                #[cfg(test)]
                event::Event::Dummy => {},
            }

            // Event got polled and handled
            return true
        }

        // Did not find or handle any events
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_run_correct() {
        let mut engine = Engine::default();
        engine.run();
        assert_eq!(engine.state(), EngineState::Running);
    }

    #[test]
    fn engine_stop_correct() {
        let mut engine = Engine::default();
        engine.run();
        engine.shutdown();
        assert_eq!(engine.state(), EngineState::Stopped);
    }
}
