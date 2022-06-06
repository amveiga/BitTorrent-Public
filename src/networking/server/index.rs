#![allow(dead_code)]

use super::{utils, Job, Protocol, Worker};
use std::collections::HashMap;
use std::io::prelude::Read;
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::str::FromStr;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Server<P: Protocol + Send + Sync + 'static> {
    socket_address: SocketAddr,
    established_connections: HashMap<SocketAddr, Worker>,
    orchestrator: Sender<Job>,
    receiver: Arc<Mutex<Receiver<Job>>>,
    protocol: P,
    port: u16,
}

impl<P: Protocol + Send + Sync + Clone + 'static> Server<P> {
    pub fn get_socket_address(&self) -> SocketAddr {
        self.socket_address
    }

    pub fn new_from_socket(protocol: P, ip: &str, port: u16) -> Self {
        Self::create_server(protocol, ip, port)
    }

    pub fn new(protocol: P) -> Self {
        let available_port = utils::get_available_port().expect("No available ports");

        Self::create_server(protocol, "127.0.0.1", available_port)
    }

    fn create_server(protocol: P, ip: &str, port: u16) -> Self {
        let established_connections: HashMap<SocketAddr, Worker> = HashMap::new();

        let (orchestrator, receiver) = channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let socket_address =
            SocketAddr::from((IpAddr::from_str(ip).expect("Couldn't parse ip"), port));

        Self {
            socket_address,
            established_connections,
            orchestrator,
            receiver,
            protocol,
            port,
        }
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }

    pub fn handle_established_connection(mut stream: TcpStream) {
        let mut buffer = [0; 1024];

        stream
            .read_exact(&mut buffer)
            .expect("Failed to read from stream");
    }

    pub fn run(&mut self) {
        let socket = TcpListener::bind(self.socket_address).expect("Listener failed to bind");

        let orchestrator = self.orchestrator.clone();
        let protocol = self.protocol.clone();
        let receiver = self.receiver.clone();
        // let established_connections = Arc::new(Mutex::new(self.established_connections.as_ref()));

        thread::spawn(move || {
            for stream in socket.incoming().flatten() {
                // unused local_address

                let _local_address = stream
                    .local_addr()
                    .expect("Failed to get local address for stream");
                let _worker = Worker::new(Arc::clone(&receiver));
                // established_connections.insert(local_address, worker);
                orchestrator
                    .send(protocol.clone().handle_incoming(stream))
                    .expect("Failed to send message to stream");
            }
        });
    }
}
