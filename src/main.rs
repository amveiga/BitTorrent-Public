use sitos::client::Client as bitClient;
use sitos::csv_env as bitEnv;
use sitos::server::Server as bitServer;

use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        4 => (),
        5 => {
            let env_type = String::from(&args[4]);
            bitEnv::set_env(env_type);
        }
        _ => {
            eprintln!(
                "Error! Wrong arguments use: 'server' / 'client' , <ip> ,<port>, optional: <ENV>"
            );
            process::exit(1);
        }
    }

    let bit_type: &str = &args[1];
    let ip = String::from(&args[2]);
    let port = String::from(&args[3]);

    match bit_type {
        "server" => {
            bitEnv::set_env(String::from("server"));
            let mut server = bitServer::start(ip, port);
            println!("Server: {}", server);

            server.run().expect("Server Failed");
            //server.stop();
        }
        "client" => {
            bitEnv::set_env(String::from("client"));
            let client = bitClient::start(ip, port);
            println!("Client: {}", client);

            client.run().expect("Client Failed");
            //client.stop();
        }
        _ => {
            eprintln!(
                "Error! Wrong arguments use: 'server' / 'client' , <ip> ,<port>, optional: <ENV>"
            );
            process::exit(1);
        }
    }
}
