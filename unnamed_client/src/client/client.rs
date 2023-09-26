use unnamed_engine::core::engine::*;

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
