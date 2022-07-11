use crate::frontend::peers::PeersData;
use crate::frontend::torrents::TorrentData;

use super::{
    BTProtocol, Bitfield, CommonInformation, Error, InterfaceProtocol, Message, NetworkingError,
    PeerList, PeerRecord, PeerState, Piece, Protocol, ServerHandler, State, BLOCK_LENGTH,
};
use std::thread;

use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct PeerConnection {
    pub peer: PeerRecord,
    client: InterfaceProtocol<BTProtocol>,
    bitfield: Arc<Mutex<Bitfield>>,
    peers: Arc<Mutex<PeerList>>,
    pub common_information: CommonInformation,
    stream: <BTProtocol as Protocol>::Stream,
    pub state: State,
    peer_state: Arc<Mutex<PeerState>>,
    pub instant: Instant,
}

impl PeerConnection {
    pub fn new(
        bitfield: Arc<Mutex<Bitfield>>,
        peers: Arc<Mutex<PeerList>>,
        common_information: CommonInformation,
        peer: PeerRecord,
        peer_state: Arc<Mutex<PeerState>>,
    ) -> Result<Self, NetworkingError> {
        let mut client = InterfaceProtocol::new(BTProtocol);

        let address = if peer.ip == ServerHandler::get_clients_ip().unwrap() {
            format!("127.0.0.1:{}", peer.port)
        } else {
            peer.get_address()
        };

        let stream = client.connect(&address)?;

        Ok(Self {
            peer_state,
            stream,
            client,
            bitfield,
            peers,
            common_information,
            state: State::UnknownToPeer,
            peer,
            instant: Instant::now(),
        })
    }

    pub fn activate(
        bitfield: Arc<Mutex<Bitfield>>,
        peers: Arc<Mutex<PeerList>>,
        common_information: CommonInformation,
        peer: PeerRecord,
        peer_state: Arc<Mutex<PeerState>>,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            match PeerConnection::new(
                bitfield,
                Arc::clone(&peers),
                common_information,
                peer.clone(),
                peer_state,
            ) {
                Ok(mut peer_connection) => loop {
                    if let PeerState::Broken = &*peer_connection.peer_state.lock().unwrap() {
                        PeersData::refresh(&peer_connection, true);
                        break;
                    }

                    let current_state = peer_connection.state.clone();

                    match current_state {
                        State::UnknownToPeer => match peer_connection.greet() {
                            Ok(new_state) => peer_connection.state = new_state,
                            _ => peer_connection.state = State::Useless(false),
                        },
                        State::ProcessingHandshakeResponse => {
                            peer_connection.state = peer_connection.process_handshake_response();
                        }
                        State::Choked => match peer_connection.unchoke() {
                            Ok(new_state) => peer_connection.state = new_state,
                            Err(_) => peer_connection.state = State::Useless(false),
                        },
                        State::Downloading => match peer_connection.download_piece() {
                            Ok(new_state) => peer_connection.state = new_state,
                            Err(_) => peer_connection.state = State::Useless(false),
                        },
                        State::Useless(_) => {
                            peers.lock().unwrap().remove(&peer.ip, peer.port);
                            PeersData::refresh(&peer_connection, true);
                            break;
                        }
                        State::FileDownloaded => {
                            break;
                        }
                    }
                },
                Err(_) => {
                    peers.lock().unwrap().remove(&peer.ip, peer.port);
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

                peers_guard.remove(&self.peer.ip, self.peer.port);
                PeersData::refresh(self, true);
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

            if self
                .client
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
                .is_err()
            {
                self.bitfield.lock().unwrap().unset_downloading(piece_index);
                return Err(Error::FailedToSavePiece);
            }

            loop {
                match Message::read_message_from_stream(&mut self.stream, false) {
                    Ok(Message::Piece { payload }) => {
                        piece.add_block(payload[8..].to_vec());
                        break;
                    }
                    Err(_) => {
                        self.bitfield.lock().unwrap().unset_downloading(piece_index);
                        return Err(Error::FailedMessageRead);
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
                let peers_guard = self.peers.lock().unwrap();

                TorrentData::refresh(&self.common_information, &peers_guard, &have_guard);
                PeersData::refresh(self, false);
                self.instant = Instant::now();

                drop(have_guard);
                log::debug!("PeerConnection::download_piece() - bitfield lock dropped");

                return Ok(State::Downloading);
            }

            self.bitfield.lock().unwrap().unset_downloading(piece_index);
            return Err(Error::FailedToSavePiece);
        }

        Err(Error::InvalidPiece)
    }
}
