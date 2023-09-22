pub struct EngineData {
    pub running: bool
}

pub trait Application {
    fn run(&self) {}
}
