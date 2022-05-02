//use sitos::client;
use sitos::server::Server as bitServer;
fn main() {
    let name = String::from("PEPE");
    let server = bitServer::new(name);
    println!("Server name: {}", server.name());
}
