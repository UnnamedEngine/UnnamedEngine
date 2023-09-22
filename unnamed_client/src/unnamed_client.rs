use unnamed_engine::core::application::*;

pub struct ClientData {
    pub engine_data: EngineData,
}

impl Application for ClientData {
    fn run(&self) {
        while self.engine_data.running {
        }
    }
}
