use unnamed_engine::core::engine::*;

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
            ||
            {

            },
            ||
            {

            },
            ||
            {

            });
    }
}
