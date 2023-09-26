mod client;

use client::client::Client;

fn main() {
    let mut client = Client::new("UnnamedClient".to_string());
    client.start();
}
