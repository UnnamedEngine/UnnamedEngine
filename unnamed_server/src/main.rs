use server::server::Server;

mod server;

fn main() {
    let mut server = Server::new("UnnamedServer".to_string());
    server.start();
}
