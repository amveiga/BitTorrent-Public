use super::{
    BTProtocol, Bitfield, Client, CommonInformation, Connections, Error, Message, NetworkingError,
    Peer, PeerList, Piece, Protocol, State, BLOCK_LENGTH,
};
use std::thread;

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

pub struct PeerConnection {
    peer: Peer,
    client: Client<BTProtocol>,
    bitfield: Arc<Mutex<Bitfield>>,
    peers: Arc<Mutex<PeerList>>,
    common_information: CommonInformation,
    stream: <BTProtocol as Protocol>::Stream,
    state: State,
}

impl PeerConnection {
    pub fn new(
        bitfield: Arc<Mutex<Bitfield>>,
        peers: Arc<Mutex<PeerList>>,
        common_information: CommonInformation,
        peer: Peer,
    ) -> Result<Self, NetworkingError> {
        let mut client = Client::new(BTProtocol);

        let stream = client.connect(&peer.get_address())?;

        Ok(Self {
            stream,
            client,
            bitfield,
            peers,
            common_information,
            state: State::UnknownToPeer,
            peer,
        })
    }

    pub fn activate(
        bitfield: Arc<Mutex<Bitfield>>,
        peers: Arc<Mutex<PeerList>>,
        common_information: CommonInformation,
        peer: Peer,
        connections: Arc<Mutex<Connections>>,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            match PeerConnection::new(
                bitfield,
                Arc::clone(&peers),
                common_information,
                peer.clone(),
            ) {
                Ok(mut peer_connection) => loop {
                    let current_state = peer_connection.state.clone();

                    match current_state {
                        State::UnknownToPeer => match peer_connection.greet() {
                            Ok(new_state) => peer_connection.state = new_state,
                            _ => {
                                break;
                            }
                        },
                        State::ProcessingHandshakeResponse => {
                            peer_connection.state = peer_connection.process_handshake_response();
                        }
                        State::Choked => match peer_connection.unchoke() {
                            Ok(new_state) => peer_connection.state = new_state,
                            Err(_) => peer_connection.state = State::Useless(false),
                        },
                        State::Downloading => match peer_connection.download_piece() {
                            Ok(new_state) => {
                                peer_connection.state = new_state;
                                log::info!("Downloaded piece")
                            }
                            Err(_) => log::info!("Error downloading piece"),
                        },
                        State::Useless(_) | State::FileDownloaded => {
                            break;
                        }
                    }

                    thread::sleep(Duration::from_secs(2));
                },
                Err(_) => {
                    peers.lock().unwrap().remove(&peer.ip);
                    connections.lock().unwrap().remove(&peer.ip);
                }
            };
        })
    }

    fn greet(&mut self) -> Result<State, NetworkingError> {
        let handshake = BTProtocol::format_handshake_message(
            &self.common_information.info_hash,
            &self.common_information.peer_id,
        );

        self.client.send(&mut self.stream, &handshake)?;

        match Message::validate_stream_handshake(
            &mut self.stream,
            handshake.len(),
            &self.common_information.info_hash,
        ) {
            Ok(true) => Ok(State::ProcessingHandshakeResponse),
            _ => {
                log::debug!("PeerConnection::greet() - Trying to obtain PeerList lock");
                let mut peers_guard = self.peers.lock().unwrap();
                log::debug!("PeerConnection::greet() - PeerList lock obtained");

                peers_guard.remove(&self.peer.ip);
                drop(peers_guard);
                log::debug!("PeerConnection::greet() - PeerList lock dropped");
                Err(NetworkingError::FailedPeerConnection)
            }
        }
    }

    fn process_handshake_response(&mut self) -> State {
        let mut state = State::Useless(false);

        loop {
            match Message::read_message_from_stream(&mut self.stream, true) {
                Ok(Message::Have { payload }) => {
                    let maybe_index = Bitfield::index_from_bytes(payload);

                    if let Ok(index) = maybe_index {
                        match state {
                            State::Downloading => {
                                self.peer.has.set(index);
                            }
                            State::Useless(unchoked) => {
                                self.peer.has.set(index);

                                match unchoked {
                                    true => {
                                        state = State::Downloading;
                                    }
                                    false => state = State::Choked,
                                }
                            }
                            State::Choked => {
                                self.peer.has.set(index);
                            }
                            _ => {}
                        }
                    } else {
                        continue;
                    }
                }
                Ok(Message::Bitfield { payload }) => match state {
                    State::Downloading => {
                        self.peer.has =
                            Bitfield::new_from_vec(payload, self.common_information.total_pieces);
                    }
                    State::Useless(unchoked) => {
                        self.peer.has =
                            Bitfield::new_from_vec(payload, self.common_information.total_pieces);

                        match unchoked {
                            true => {
                                state = State::Downloading;
                            }
                            false => state = State::Choked,
                        }
                    }
                    State::Choked => {
                        self.peer.has =
                            Bitfield::new_from_vec(payload, self.common_information.total_pieces);
                    }
                    _ => {}
                },
                Ok(Message::Unchoke) => match state {
                    State::Useless(_unchoked) => state = State::Useless(true),
                    State::Choked => {
                        state = State::Downloading;
                    }
                    _ => {}
                },
                _ => {
                    break;
                }
            }
        }

        state
    }

    fn unchoke(&mut self) -> Result<State, Error> {
        self.client
            .send(
                &mut self.stream,
                Message::Interested
                    .parse()
                    .expect("Failed to parse interested message"),
            )
            .unwrap();

        match Message::read_message_from_stream(&mut self.stream, false) {
            Ok(Message::Unchoke) => Ok(State::Downloading),
            _ => Err(Error::FailedToUnchoke),
        }
    }

    fn download_piece(&mut self) -> Result<State, Error> {
        log::debug!("PeerConnection::download_piece() - trying to obtain bitfield lock");
        let mut have_guard = self.bitfield.lock().unwrap();
        log::debug!("PeerConnection::download_piece() - bitfield lock obtained");

        let maybe_piece_index = self.peer.has.first_needed_available_piece(&have_guard);

        if maybe_piece_index == None && have_guard.is_complete() {
            return Ok(State::FileDownloaded);
        }

        let piece_index = maybe_piece_index.ok_or(Error::NoNewPiecesFromPeer)?;

        have_guard.set_downloading(piece_index);

        drop(have_guard);
        log::debug!("PeerConnection::download_piece() - bitfield lock dropped");

        let piece_length = if piece_index == self.common_information.total_pieces - 1 {
            let total_pieces = (self.common_information.file_length as f64
                / self.common_information.piece_length as f64)
                .ceil() as usize;

            (self.common_information.file_length
                - (self.common_information.piece_length * (total_pieces - 1)) as u64)
                as usize
        } else {
            self.common_information.piece_length
        };

        let mut piece = Piece::new(piece_index, piece_length, BLOCK_LENGTH as usize);

        while !piece.is_complete() {
            log::debug!("PeerConnection::download_piece() - trying to download piece");
            let (piece_index, block_offset, block_length) = piece.get_next_block_attributes();

            self.client
                .send(
                    &mut self.stream,
                    Message::Request {
                        piece_index: piece_index as u32,
                        block_offset: block_offset as u32,
                        block_length: block_length as u32,
                    }
                    .parse()
                    .expect("Failed to parse request message"),
                )
                .unwrap();

            loop {
                match Message::read_message_from_stream(&mut self.stream, false) {
                    Ok(Message::Piece { payload }) => {
                        piece.add_block(payload[8..].to_vec());
                        break;
                    }
                    Err(_) => {
                        break;
                    }
                    _ => {}
                }
            }
        }

        if piece.verify(self.common_information.pieces[piece_index].clone()) {
            log::info!("Piece {} verified", piece_index);
            if piece.save(&self.common_information.file_name).is_ok() {
                log::info!("Piece {} saved", piece_index);
                log::debug!("PeerConnection::download_piece() - trying to obtain bitfield lock");
                let mut have_guard = self.bitfield.lock().unwrap();
                log::debug!("PeerConnection::download_piece() - bitfield lock obtained");
                have_guard.set(piece_index);
                drop(have_guard);
                log::debug!("PeerConnection::download_piece() - bitfield lock dropped");
                return Ok(State::Downloading);
            }
            return Err(Error::FailedToSavePiece);
        }

        Err(Error::InvalidPiece)
    }
}
