use strum::Display;

/// All the possible states the `Engine` can be at.
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

pub struct EngineData {
    /// Flags the current state of the `Engine`. This value will be read a lot
    /// and rarely will change.
    pub state: EngineState,
}

/// Contains the major data required to run the application.
pub struct Engine {
    data: EngineData,
}

impl Default for Engine {
    fn default() -> Self {
        // The logger is started here to make sure we have logging always
        // available

        // Read the env values that configure the logger
        let env = env_logger::Env::default()
            .filter_or("MY_LOG_LEVEL", "info")
            .write_style_or("MY_LOG_STYLE", "always");

        // Initialize the logger from the values
        // Now we can use `log::` everywhere without worrying
        env_logger::init_from_env(env);

        let data = EngineData {
            state: EngineState::Stopped,
        };

        Self {
            data,
        }
    }
}

impl Engine {
    /// Creates a new `Engine` using `Default`.
    pub fn new() -> Self {
        Self::default()
    }

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

    /// Internal function that handles the `Engine` starting.
    fn start(&mut self) {
        self.data.state = EngineState::Running;
        log::info!("Successfully started engine");
    }

    /// Internal function that handles the `Engine` stopping.
    fn stop(&mut self) {
        self.data.state = EngineState::Stopped;
        log::info!("Successfully stopped engine");
    }

    /// Gets the current `EngineState`.
    pub fn state(&self) -> EngineState {
        self.data.state.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper method that creates and starts an `Engine`.
    fn create_and_start() -> Engine {
        let mut engine = Engine::new();
        engine.run();
        engine
    }

    #[test]
    fn engine_run_correct() {
        let engine = create_and_start();
        assert_eq!(engine.state(), EngineState::Running);
    }

    #[test]
    fn engine_stop_correct() {
        let mut engine = create_and_start();
        engine.shutdown();
        assert_eq!(engine.state(), EngineState::Stopped);
    }
}
