use super::Types;
use std::collections::LinkedList;

pub struct Peer {
    pub peer_id: Vec<u8>,
    pub ip: String,
    pub port: i64,
    pub has: Vec<u8>,
}

impl Peer {
    pub fn new_from_list(list: &LinkedList<Types>) -> Vec<Self> {
        let mut peer_list: Vec<Self> = Vec::new();

        list.iter().for_each(|peer| {
            if let Types::Dictionary(peer) = peer {
                let peer_id = peer
                    .get(&b"peer id".to_vec())
                    .expect("Failed to parse peer_id for peer");

                let ip = peer
                    .get(&b"ip".to_vec())
                    .expect("Failed to parse ip for peer");

                let port = peer
                    .get(&b"port".to_vec())
                    .expect("Failed to parse port for peer");

                if let (Types::String(peer_id), Types::String(ip), Types::Integer(port)) =
                    (peer_id, ip, port)
                {
                    peer_list.push(Self {
                        has: Vec::new(),
                        port: *port,
                        peer_id: peer_id.clone(),
                        ip: String::from_utf8(ip.clone()).expect("Malformed peer ip"),
                    });
                }
            };
        });

        peer_list
    }
}
