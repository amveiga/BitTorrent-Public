#![allow(unused_variables)]

pub use super::{Job, Protocol};
use native_tls::{TlsConnector, TlsStream};
use std::collections::HashMap;
use std::convert::AsRef;
use std::io::prelude::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

pub struct HTTPSTracker {
    peers: Vec<usize>,
    announce: String,
}

impl Clone for HTTPSTracker {
    fn clone(&self) -> Self {
        Self {
            peers: self.peers.clone(),
            announce: self.announce.clone(),
        }
    }
}

impl HTTPSTracker {
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

    pub fn format_handshake_message(
        &self,
        address: &str,
        id: String,
        info_hash: String,
        port: u16,
        left: u64,
        downloaded: u64,
    ) -> String {
        format!(
            "GET /{}?peer_id={}&numwant=100&info_hash={}&port={}&uploaded=0&downloaded={}&left={}&event=started HTTP/1.0\r\nHost: {}\r\n\r\n",
            self.announce,id, info_hash, port, downloaded, left, address
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

impl Protocol for HTTPSTracker {
    type Stream = TlsStream<TcpStream>;

    fn connect(target_address: &str) -> Result<TlsStream<TcpStream>, String> {
        let connector = TlsConnector::new().expect("Failed to creat TlsConnector");
        let split: Vec<&str> = target_address.split(':').collect();

        log::info!(
            "HTTPSTracker::connect() - Trying to connect to {}",
            target_address
        );
        match TcpStream::connect(format!("{}:443", split[0])) {
            Ok(stream) => match connector.connect(split[0], stream) {
                Ok(stream) => {
                    log::info!(
                        "HTTPSTracker::connect() - Successfully connected to {}",
                        target_address
                    );
                    Ok(stream)
                }
                Err(_) => {
                    log::error!(
                        "HTTPSTracker::connect() - Error connecting to {}",
                        target_address
                    );
                    Err(String::from("Failed to connect to TlsConnector"))
                }
            },
            Err(_) => {
                log::error!(
                    "HTTPSTracker::connect() - Error connecting to {}",
                    target_address
                );
                Err(format!("Failed to connect to {}", target_address))
            }
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
        })
    }

    fn handle_request<R: AsRef<[u8]>>(&self, request: R, mut stream: TcpStream) -> Job {
        let bytes = request.as_ref().to_owned();

        Box::new(move || {
            stream.write_all(&bytes).expect("Failed to write in stream");
        })
    }
}
