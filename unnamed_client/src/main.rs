use unnamed_engine::core::engine::Engine;

fn main() {
    let mut engine = Engine::default();
    engine.run();
    engine.shutdown();
}
