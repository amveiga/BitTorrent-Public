#![allow(dead_code)]

use super::{
    constants::{BLOCK_LENGTH, BLOCK_LENGTH_B},
    utils::{random_u64_as_bytes, split_u8},
    BTProtocol, Client, Decoder, File, HTTPTracker, Message, Peer, Role, Server, Torrent, Types,
    UrlEncoder,
};
use sha1::{Digest, Sha1};

pub struct Leecher {
    have: Vec<u8>,
    peer_id: Vec<u8>,
    tracker_address: String,
    tracker_side: Client<HTTPTracker>,
    client_side: Client<BTProtocol>,
    server_side: Server<BTProtocol>,
    torrent: Torrent,
    peers: Vec<Peer>,
}

impl Role for Leecher {
    fn new(torrent_pathname: &str) -> Self {
        let torrent = Torrent::new_from_pathname(torrent_pathname);

        let announce = String::from_utf8(torrent.get_announce().expect("Failed to get announce"))
            .expect("Failed to parse announce")
            .replace("https://", "");

        let split: Vec<&str> = announce.split('/').collect();

        let announce_base = format!("{}:443", split[0]);
        let announce_endpoint = format!("/{}", String::from(split[1]));

        let mut tracker_side = Client::new(HTTPTracker::new(Some(announce_endpoint)));

        if tracker_side.connect(&announce_base).is_err() {
            panic!("Tracker not connecting")
        };

        let mut server_side = Server::new(BTProtocol);

        server_side.run();

        let mut hasher = Sha1::new();
        hasher.update(random_u64_as_bytes());

        let peer_id: [u8; 20] = hasher.finalize().into();

        let file_length = torrent
            .get_length()
            .expect("Malformed length in torrent file");

        let bitfield_length = ((file_length as f64) / 8.0).ceil();

        Self {
            have: vec![0; bitfield_length as usize],
            tracker_address: announce_base,
            peers: Vec::new(),
            torrent,
            tracker_side,
            server_side,
            peer_id: peer_id.to_vec(),
            client_side: Client::new(BTProtocol),
        }
    }

    // desconectar todos los client/servers

    fn deactivate(&mut self) {}

    fn activate(&mut self) {
        let info_hash = self.torrent.get_info_hash();
        let length = self.torrent.get_length().expect("No length available");

        let request = HTTPTracker::send_handshake(
            &self.tracker_address,
            UrlEncoder::encode_binary_data(self.peer_id.clone()),
            UrlEncoder::encode_binary_data(info_hash.to_vec()),
            self.server_side.get_port(),
            length,
        );

        // Hay que hacer esto en loop en un thread
        // Tiene que requestear cada X cantidad de tiempo un update al trcker
        // Hay que fijarse de NO reemplazar las keys existentes en las established_connections

        let response = self
            .tracker_side
            .send(&self.tracker_address, request)
            .unwrap();

        let slice = split_u8(response, b"\r\n\r\n");

        if let Types::Dictionary(dict) = Decoder::new_from_bytes(&slice)
            .decode()
            .expect("Failed to parse tracker response")
        {
            let peers = dict
                .get(&b"peers".to_vec())
                .expect("Failed to parse peers from tracker response");

            if let Types::List(list) = peers {
                let peers = Peer::new_from_list(list);
                self.peers = peers;
            } else {
                println!("No peers available");
            }
        };

        // Voy a probar conectando a un solo peer
        // Sistema de retries para los peers

        let handshake = BTProtocol::handshake(&info_hash, &self.peer_id);

        let mut peer_in_use_index = 0;

        let (peer_in_use, peer_address) = loop {
            let peer_in_use = self
                .peers
                .get_mut(peer_in_use_index)
                .expect("No peer found");
            let peer_address = format!("{}:{}", peer_in_use.ip, peer_in_use.port);

            match self
                .client_side
                .connect(&format!("{}:{}", peer_in_use.ip, peer_in_use.port))
            {
                Ok(()) => {
                    println!("Successfully connected to peer: {}", peer_address);
                    break (peer_in_use, peer_address);
                }
                _ => {
                    println!("Unsuccessful connection with peer: {}", peer_address);
                    peer_in_use_index += 1;
                    continue;
                }
            };
        };

        println!("Waiting handshake");

        let message_stream = self.client_side.send_stream(&peer_address, &handshake);

        match Message::validate_stream_handshake(message_stream, handshake.len(), &info_hash) {
            Ok(true) => {
                println!("Valid handshake");

                let maybe_bitfield = Message::read_message_from_stream(message_stream);

                if let Ok(Message::Bitfield { payload }) = maybe_bitfield {
                    println!("Got bitfield");

                    peer_in_use.has = payload;

                    let message_stream = self.client_side.send_stream(
                        &peer_address,
                        Message::Interested
                            .parse()
                            .expect("Failed to parse interested message"),
                    );

                    match Message::read_message_from_stream(message_stream) {
                        Ok(Message::Unchoke) => {
                            // Implementar Piece

                            let pieces = self
                                .torrent
                                .get_pieces()
                                .expect("Corrupted torrent, no pieces attribute");

                            let file_name = String::from_utf8_lossy(
                                &self
                                    .torrent
                                    .get_name()
                                    .expect("Corrupted torrent, no name attribute"),
                            )
                            .to_string();

                            let piece_length = self
                                .torrent
                                .get_piece_length()
                                .expect("Corrupted torrent, no piece length");

                            let num_blocks =
                                ((piece_length / (BLOCK_LENGTH as i64)) as f64).ceil() as usize;

                            let last_block_len = (piece_length
                                - (BLOCK_LENGTH as i64) * ((num_blocks as i64) - 1))
                                as u32;

                            let mut piece = vec![];

                            let mut block_index: usize = 0;
                            let mut block_offset: u32 = 0;

                            while block_index < num_blocks {
                                println!("block index {}", block_index);

                                let mut request_payload = vec![0, 0, 0, 0];
                                request_payload.extend_from_slice(&block_offset.to_be_bytes());

                                if block_index == num_blocks - 1 {
                                    request_payload
                                        .extend_from_slice(&last_block_len.to_be_bytes());
                                } else {
                                    request_payload.extend_from_slice(&BLOCK_LENGTH_B);
                                }

                                let message_stream = self.client_side.send_stream(
                                    &peer_address,
                                    Message::Request {
                                        payload: request_payload,
                                    }
                                    .parse()
                                    .expect("Failed to parse request message"),
                                );

                                match Message::read_message_from_stream(message_stream) {
                                    Ok(Message::Piece { payload }) => {
                                        piece.extend_from_slice(&payload[8..]);
                                        block_index += 1;
                                        block_offset = (block_index as u32) * BLOCK_LENGTH;
                                    }
                                    Err(_) => {}
                                    _ => {}
                                }
                            }

                            match File::new_file_from_piece(&piece, format!("{}.piece0", file_name))
                            {
                                Ok(_) => {
                                    println!("Saved piece correctly");

                                    let mut sha_encoder = Sha1::new();

                                    sha_encoder.update(&piece);
                                    let piece_hash: [u8; 20] = sha_encoder.finalize().into();

                                    if piece_hash.to_vec() == pieces[0] {
                                        println!("The piece is valid, sha verified")
                                    } else {
                                        println!("The piece is not valid, sha verified")
                                    }
                                }
                                Err(_) => {
                                    println!("Failed to save piece");
                                }
                            };
                        }
                        Err(_) => {}
                        _ => {}
                    }
                }
            }
            _ => {
                println!("Invalid handshake");
            }
        }
    }
}
