use unnamed_client::ClientData;
use unnamed_engine::core::application::{EngineData, Application};

mod unnamed_client;

fn main() {
    let client_data = ClientData {
        engine_data: EngineData {
            running: true
        }
    };

    client_data.run();
}
