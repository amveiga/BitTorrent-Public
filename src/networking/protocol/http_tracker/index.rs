pub use super::Protocol;
use std::collections::HashMap;
use std::net::TcpStream;

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
            self.announce,id, info_hash, port, downloaded,left, address
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
    type Stream = TcpStream;

    fn connect(target_address: &str) -> Result<TcpStream, String> {
        match TcpStream::connect(target_address) {
            Ok(stream) => Ok(stream),
            Err(_) => Err(format!("Failed to connect to {}", target_address)),
        }
    }
}
