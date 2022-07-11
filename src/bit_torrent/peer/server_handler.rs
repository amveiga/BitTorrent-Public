use super::{
    networking::utils::get_available_port, Bitfield, CommonInformation, PeerState, ServerConnection,
};
use std::thread::{self};

use std::net::TcpListener;
use std::process::Command;
use std::sync::{Arc, Mutex};

pub struct ServerHandler {
    bitfield: Arc<Mutex<Bitfield>>,
    peer_state: Arc<Mutex<PeerState>>,
    common_information: CommonInformation,
    ip: String,
    port: u16,
    socket: TcpListener,
}

impl ServerHandler {
    pub fn get_clients_ip() -> Result<String, ()> {
        if let Ok(output) = Command::new("dig")
            .args([
                "-4",
                "TXT",
                "+short",
                "o-o.myaddr.l.google.com",
                "@ns1.google.com",
            ])
            .output()
        {
            let address = String::from_utf8(output.stdout).unwrap();
            let address = address.split('"').collect::<Vec<&str>>()[1];
            let address = String::from(address).replace('\\', "");

            return Ok(address);
        }

        Err(())
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }

    pub fn get_ip(&self) -> String {
        self.ip.clone()
    }

    pub fn new(
        bitfield: Arc<Mutex<Bitfield>>,
        common_information: CommonInformation,
        peer_state: Arc<Mutex<PeerState>>,
    ) -> Result<Self, ()> {
        let maybe_port = get_available_port();

        if let Some(port) = maybe_port {
            let address = format!("{}:{}", "127.0.0.1", port);

            let socket = TcpListener::bind(address).or(Err(()))?;

            return Ok(Self {
                socket,
                peer_state,
                ip: ServerHandler::get_clients_ip().expect("Failed to get global ip"),
                port,
                bitfield,
                common_information,
            });
        }

        *peer_state.lock().unwrap() = PeerState::Broken;
        Err(())
    }

    pub fn activate(self) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let connections: Vec<thread::JoinHandle<()>> = vec![];

            self.socket.set_nonblocking(true).unwrap();

            for maybe_stream in self.socket.incoming() {
                match maybe_stream {
                    Ok(stream) => {
                        ServerConnection::activate(
                            Arc::clone(&self.bitfield),
                            Arc::clone(&self.peer_state),
                            self.common_information.clone(),
                            stream,
                        );
                    }
                    _ => {
                        if let PeerState::Broken = &*self.peer_state.lock().unwrap() {
                            for connection in connections {
                                connection.join().unwrap();
                            }
                            break;
                        }
                    }
                }
            }
        })
    }
}
