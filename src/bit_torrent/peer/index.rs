use super::{Bitfield, Types};
use std::collections::LinkedList;
use std::net::Ipv6Addr;

#[derive(Clone, Debug)]
pub struct Peer {
    pub ip: String,
    pub port: i64,
    pub has: Bitfield,
    pub ipv6: bool,
    pub in_use: bool,
}

impl Peer {
    pub fn get_address(&self) -> String {
        match self.ipv6 {
            true => {
                let addr: Ipv6Addr = self.ip.parse().expect("Failed to parse ipv6 addr");

                format!("{}:{}", addr, self.port)
            }
            false => format!("{}:{}", self.ip, self.port),
        }
    }

    pub fn new_from_list(list: &LinkedList<Types>, total_pieces: usize) -> Vec<Self> {
        let mut peer_list: Vec<Self> = Vec::new();

        list.iter().for_each(|peer| {
            if let Types::Dictionary(peer) = peer {
                let ip = peer
                    .get(&b"ip".to_vec())
                    .expect("Failed to parse ip for peer");

                let port = peer
                    .get(&b"port".to_vec())
                    .expect("Failed to parse port for peer");

                if let (Types::String(ip), Types::Integer(port)) = (ip, port) {
                    let ip = String::from_utf8(ip.clone()).expect("Malformed peer ip");

                    peer_list.push(Self {
                        in_use: false,
                        ipv6: ip.contains(':'),
                        has: Bitfield::new(total_pieces),
                        port: *port,
                        ip,
                    });
                }
            };
        });

        peer_list
    }
}
