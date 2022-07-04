#![allow(unused_mut)]
#![allow(unused_variables)]

use super::{Job, Protocol};
use std::convert::AsRef;
use std::net::TcpStream;

pub struct BitTorrent;

impl Clone for BitTorrent {
    fn clone(&self) -> Self {
        Self {}
    }
}

impl BitTorrent {
    pub fn format_handshake_message(info_hash: &[u8], peer_id: &[u8]) -> Vec<u8> {
        let mut handshake: Vec<u8> = Vec::new();

        let handshake_protocol = b"BitTorrent protocol".to_vec();
        let handshake_protocol_length = handshake_protocol.len() as u8;
        let reserved = [0_u8; 8];

        handshake.push(handshake_protocol_length);
        handshake.extend_from_slice(&handshake_protocol);
        handshake.extend_from_slice(&reserved);
        handshake.extend_from_slice(info_hash);
        handshake.extend_from_slice(peer_id);

        handshake
    }
}

impl Protocol for BitTorrent {
    type Stream = TcpStream;

    fn connect(target_address: &str) -> Result<TcpStream, String> {
        log::info!(
            "BitTorrent::connect() - Trying to connect to {}",
            target_address
        );
        match TcpStream::connect(target_address) {
            Ok(stream) => {
                log::info!(
                    "BitTorrent::connect() - Successfully connected to {}",
                    target_address
                );

                Ok(stream)
            }
            Err(_) => {
                log::error!(
                    "BitTorrent::connect() - Error connecting to {}",
                    target_address
                );

                Err(format!("Failed to connect to {}", target_address))
            }
        }
    }

    fn handle_request<R: AsRef<[u8]>>(&self, _request: R, _stream: TcpStream) -> Job {
        Box::new(move || {})
    }

    fn handle_incoming(self, mut stream: TcpStream) -> Job {
        Box::new(move || {})
    }
}
