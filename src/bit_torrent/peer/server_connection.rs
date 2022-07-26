use super::{file_system::File, Bitfield, CommonInformation, Message, PeerState, UploadState};
use std::io::Write;
use std::net::TcpStream;
use std::thread;

use std::sync::{Arc, Mutex};

pub struct ServerConnection {
    bitfield: Arc<Mutex<Bitfield>>,
    peer_state: Arc<Mutex<PeerState>>,
    common_information: CommonInformation,
    state: UploadState,
}

impl ServerConnection {
    fn new(
        bitfield: Arc<Mutex<Bitfield>>,
        peer_state: Arc<Mutex<PeerState>>,
        common_information: CommonInformation,
    ) -> Self {
        Self {
            peer_state,
            state: UploadState::UnknownPeer,
            bitfield,
            common_information,
        }
    }

    pub fn activate(
        bitfield: Arc<Mutex<Bitfield>>,
        peer_state: Arc<Mutex<PeerState>>,
        common_information: CommonInformation,
        mut stream: TcpStream,
    ) -> thread::JoinHandle<()> {
        let mut connection = Self::new(bitfield, peer_state, common_information);

        thread::spawn(move || loop {
            let state_guard = connection.peer_state.lock().unwrap();

            if let PeerState::Broken = &*state_guard {
                break;
            }

            drop(state_guard);

            let current_state = connection.state.clone();

            match current_state {
                UploadState::UnknownPeer => {
                    if let Ok(new_state) =
                        Self::validate_peer(&mut stream, connection.common_information.peer_id)
                    {
                        connection.state = new_state;
                    } else {
                        connection.state = UploadState::Useless;
                    }
                }
                UploadState::AwaitingResponse => {
                    if let Ok(new_state) = connection.send_handshake_response(&mut stream) {
                        connection.state = new_state;
                    } else {
                        connection.state = UploadState::Useless;
                    }
                }
                UploadState::Uploading => {
                    if let Ok(new_state) = connection.serve_file(&mut stream) {
                        connection.state = new_state;
                    } else {
                        connection.state = UploadState::Useless;
                    }
                }
                UploadState::Useless => {
                    break;
                }
            }
        })
    }

    fn serve_file(&self, stream: &mut TcpStream) -> Result<UploadState, ()> {
        if let Ok(Message::Request {
            piece_index,
            block_offset,
            block_length,
        }) = Message::read_message_from_stream(stream, false)
        {
            let bitfield_guard = self.bitfield.lock().unwrap();

            if bitfield_guard.has(piece_index as usize) {
                let state_guard = self.peer_state.lock().unwrap();

                let maybe_block = match &*state_guard {
                    PeerState::SomePieces(pathname) => {
                        let mut file = File::new(format!(
                            "{}/{}.piece{}",
                            pathname, self.common_information.file_name, piece_index
                        ));

                        Some(file.get_block(
                            0,
                            self.common_information.piece_length,
                            block_length as usize,
                            block_offset as usize,
                        ))
                    }
                    PeerState::AllPieces(pathname) => {
                        let mut file = File::new(pathname);

                        Some(file.get_block(
                            piece_index as usize,
                            self.common_information.piece_length,
                            block_length as usize,
                            block_offset as usize,
                        ))
                    }
                    _ => None,
                };

                if let Some(block) = maybe_block {
                    let mut message = vec![];

                    message.extend_from_slice(&piece_index.to_be_bytes());
                    message.extend_from_slice(&block_offset.to_be_bytes());
                    message.extend_from_slice(&block);

                    let response = Message::Piece { payload: message };
                    stream
                        .write_all(&response.parse().expect("Failed to parse piece message"))
                        .or(Err(()))?;
                }
            }
        };

        Ok(UploadState::Uploading)
    }

    fn send_handshake_response(&self, stream: &mut TcpStream) -> Result<UploadState, ()> {
        let message = Message::Unchoke.parse().ok_or(())?;

        stream.write_all(&message).or(Err(()))?;
        stream.flush().or(Err(()))?;

        let bitfield_guard = self.bitfield.lock().unwrap();
        let payload = bitfield_guard.get();

        drop(bitfield_guard);

        stream
            .write_all(
                &Message::Bitfield { payload }
                    .parse()
                    .expect("Failed to parse bitfield"),
            )
            .or(Err(()))?;

        stream.flush().or(Err(()))?;

        Ok(UploadState::Uploading)
    }

    fn validate_peer(stream: &mut TcpStream, own_peer_id: [u8; 20]) -> Result<UploadState, ()> {
        match Message::read_handshake_from_stream(stream) {
            Ok(Message::Handshake(info_hash, _)) => {
                stream
                    .write_all(
                        &Message::HandshakeResponse(info_hash, own_peer_id)
                            .parse()
                            .expect("Failed to parse handshake response"),
                    )
                    .or(Err(()))?;
                Ok(UploadState::AwaitingResponse)
            }
            _ => Err(()),
        }
    }
}
