pub use super::{Handshake, Protocol};
use native_tls::{TlsConnector, TlsStream};
use std::collections::HashMap;
use std::net::TcpStream;

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

    pub fn format_handshake_message(&self, handshake_params: Handshake) -> String {
        format!(
            "GET /{}?peer_id={}&numwant=100&info_hash={}&port={}&uploaded=0&downloaded={}&left={}&event={} HTTP/1.0\r\nHost: {}\r\n\r\n",
            self.announce,
            handshake_params.id,
            handshake_params.info_hash,
            handshake_params.port,
            handshake_params.downloaded,
            handshake_params.left,
            handshake_params.event,
            handshake_params.address
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
}
