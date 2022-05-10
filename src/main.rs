use sitos::client::Client as bitClient;
use sitos::server::Server as bitServer;

use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Error! Wrong arguments use: 'server' / 'client' , <ip> ,<port>");
        process::exit(1);
    }

    let bit_type: &str = &args[1];
    let ip = String::from(&args[2]);
    let port = String::from(&args[3]);

    match bit_type {
        "server" => {
            let mut server = bitServer::start(ip, port);
            println!("Server: {}", server);

            server.run().expect("Server Failed");
            //server.stop();
        }
        "client" => {
            let client = bitClient::start(ip, port);
            println!("Client: {}", client);

            client.run().expect("Client Failed");
            //client.stop();
        }
        _ => {
            eprintln!("Error! Wrong arguments use: 'server' / 'client' , <ip> ,<port>");
            process::exit(1);
        }
    }
}
