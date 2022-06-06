#![allow(unused_variables)]

pub use super::{Job, Protocol};
use native_tls::{TlsConnector, TlsStream};
use std::collections::HashMap;
use std::convert::AsRef;
use std::io::prelude::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

pub struct HTTPTracker {
    peers: Vec<usize>,
    announce: String,
}

impl Clone for HTTPTracker {
    fn clone(&self) -> Self {
        Self {
            peers: self.peers.clone(),
            announce: self.announce.clone(),
        }
    }
}

impl HTTPTracker {
    pub fn new(announce: Option<String>) -> Self {
        match announce {
            Some(announce) => Self {
                peers: Vec::new(),
                announce,
            },
            None => Self {
                peers: Vec::new(),
                announce: String::from("/announce"),
            },
        }
    }

    pub fn handshake_message_format(
        address: &str,
        id: String,
        info_hash: String,
        port: u16,
        left: i64,
    ) -> String {
        format!(
            "GET /announce?peer_id={}&info_hash={}&port={}&uploaded=0&downloaded=0&left={}&event=started HTTP/1.0\r\nHost: {}\r\n\r\n",
            id, info_hash, port, left, address
        )
    }

    pub fn query_to_hashmap(query: &str) -> HashMap<String, String> {
        let mut map: HashMap<String, String> = HashMap::new();

        let query_params: Vec<&str> = query.split('?').collect();
        let query_params = query_params[1];

        query_params.split('&').into_iter().for_each(|key_value| {
            let key_value_split: Vec<&str> = key_value.split('=').collect();

            let key = key_value_split[0];
            let value = key_value_split[1];

            map.insert(String::from(key), String::from(value));
        });

        map
    }

    pub fn hasmap_to_query(map: HashMap<String, String>) -> String {
        let key_values: Vec<String> = map
            .iter()
            .map(|(key, value)| format!("{}={}", key, value))
            .collect();

        format!("?{}", key_values.join("&"))
    }

    pub fn add_peer(&mut self, peer_id: usize) {
        self.peers.push(peer_id);
    }
}

impl Protocol for HTTPTracker {
    type Stream = TlsStream<TcpStream>;

    fn connect(target_address: &str) -> Result<TlsStream<TcpStream>, String> {
        let connector = TlsConnector::new().expect("Failed to creat TlsConnector");
        let split: Vec<&str> = target_address.split(':').collect();

        match TcpStream::connect(format!("{}:443", split[0])) {
            Ok(stream) => match connector.connect(split[0], stream) {
                Ok(stream) => Ok(stream),
                Err(_) => Err(String::from("Failed to connect to TlsConnector")),
            },
            Err(_) => Err(format!("Failed to connect to {}", target_address)),
        }
    }

    fn handle_incoming(self, mut stream: TcpStream) -> Job {
        let mut buffer = [0; 1024];

        stream
            .read_exact(&mut buffer)
            .expect("Failed to read from stream");

        let self_pointer = Arc::new(Mutex::new(self));

        Box::new(move || {
            let string_request = String::from_utf8_lossy(&buffer).to_string();

            let request_body: Vec<&str> = string_request.split("\r\n").collect();

            let method = request_body
                .get(0)
                .expect("Failed to get method from request body");
            let url_query = request_body
                .get(1)
                .expect("Failed to get url_query from request body");

            // match *method {
            //     "GET" => {
            //         let mut self_pointer = self_pointer.lock().expect("Failed to lock Tracker");

            //         let map = HTTPTracker::query_to_hashmap(url_query);

            //         let peer_id = map.get("peer_id");

            //         if let Some(new_peer) = peer_id {
            //             let new_peer: usize = new_peer.parse().expect("Failed to parse peer");
            //             self_pointer.add_peer(new_peer);
            //         }

            //         stream
            //             .write_all(format!("{:?}", self_pointer.peers.clone()).as_bytes())
            //             .expect("Failed to write");
            //     }
            //     _ => {}
            // };
        })
    }

    fn handle_request<R: AsRef<[u8]>>(&self, request: R, mut stream: TcpStream) -> Job {
        let bytes = request.as_ref().to_owned();

        Box::new(move || {
            stream.write_all(&bytes).expect("Failed to write in stream");
        })
    }
}
