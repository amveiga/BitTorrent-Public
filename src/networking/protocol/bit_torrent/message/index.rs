use super::MessageError;
use std::io::Read;
use std::net::TcpStream;

#[derive(Debug)]

pub enum Message {
    Handshake(Vec<u8>, Vec<u8>),
    HandshakeResponse(Vec<u8>, [u8; 20]),
    Unrecognized,
    KeepAlive,
    Interested,
    Unchoke,
    Request {
        piece_index: u32,
        block_offset: u32,
        block_length: u32,
    },
    Bitfield {
        payload: Vec<u8>,
    },
    Piece {
        payload: Vec<u8>,
    },
    Have {
        payload: Vec<u8>,
    },
}

impl Message {
    pub fn validate_stream_handshake(
        stream: &mut TcpStream,
        handshake_len: usize,
        info_hash: &[u8],
    ) -> Result<bool, String> {
        let mut response = vec![0_u8; handshake_len];

        match stream.read_exact(&mut response) {
            Ok(_) => {
                if response.len() == handshake_len
                    && response[handshake_len - 40..handshake_len - 20].starts_with(info_hash)
                {
                    return Ok(true);
                }
                Ok(false)
            }
            Err(_) => Err(String::from("Failed to read handshake from stream")),
        }
    }

    pub fn read_handshake_from_stream(stream: &mut TcpStream) -> Result<Message, MessageError> {
        let mut protocol_length_buffer = [0_u8; 1];

        match stream.read_exact(&mut protocol_length_buffer) {
            Ok(_) => {
                let protocol_length = u8::from_be_bytes(protocol_length_buffer) as usize;

                let data_length = (protocol_length + 48) as usize;

                let mut data_buffer = vec![0_u8; data_length];

                match stream.read_exact(&mut data_buffer) {
                    Ok(_) => {
                        let info_hash = data_buffer[data_length - 40..data_length - 20].to_vec();
                        let peer_id = data_buffer[data_length - 20..data_length].to_vec();

                        Ok(Message::Handshake(info_hash, peer_id))
                    }
                    Err(_) => Err(MessageError::InvalidHandshake),
                }
            }
            Err(_) => Err(MessageError::FailedToReadMessage),
        }
    }

    pub fn read_message_from_stream(
        stream: &mut TcpStream,
        non_blocking: bool,
    ) -> Result<Message, String> {
        let mut length_buffer = [0_u8; 4];
        let mut id_buffer = [0_u8; 1];

        if non_blocking && stream.set_nonblocking(true).is_err() {
            return Err(String::from("Failed to set stream as nonblocking"));
        }

        let maybe_message = match stream.read_exact(&mut length_buffer) {
            Ok(_) => {
                if non_blocking && stream.set_nonblocking(false).is_err() {
                    return Err(String::from("Failed to set stream as blocking"));
                }

                let payload_length = u32::from_be_bytes(length_buffer);

                if payload_length == 0 {
                    return Ok(Self::new(length_buffer.to_vec()));
                }

                match stream.read_exact(&mut id_buffer) {
                    Ok(_) => {
                        let mut message = vec![];
                        message.extend_from_slice(&length_buffer);
                        message.extend_from_slice(&id_buffer);

                        let payload_length = payload_length - 1;

                        if payload_length == 0 {
                            return Ok(Self::new(message));
                        }

                        let mut payload_buffer = vec![0_u8; payload_length as usize];

                        match stream.read_exact(&mut payload_buffer) {
                            Ok(_) => {
                                message.extend_from_slice(&payload_buffer);
                                Ok(Self::new(message))
                            }
                            Err(_) => Err(String::from("Failed to read payload from stream")),
                        }
                    }
                    Err(_) => Err(String::from("Failed to read id from stream")),
                }
            }
            Err(_) => Err(String::from("Failed to read length from stream")),
        };

        if non_blocking && stream.set_nonblocking(false).is_err() {
            return Err(String::from("Failed to set stream as blocking"));
        }

        maybe_message
    }

    pub fn read_length_from_header(header: &[u8; 5]) -> u32 {
        u32::from_be_bytes(header[0..4].try_into().expect("Incorrect message length")) - 1
    }

    pub fn parse(&self) -> Option<Vec<u8>> {
        match self {
            Message::KeepAlive => Some(vec![0]),
            Message::Interested => Some(vec![0, 0, 0, 1, 2]),
            Message::Unrecognized => None,
            Message::Unchoke => Some(vec![0, 0, 0, 1, 1]),
            Message::Bitfield { payload } => {
                let mut message = vec![];

                let payload_length = payload.len();

                message.extend_from_slice(&((payload_length + 1) as u32).to_be_bytes());
                message.extend_from_slice(&(5_u8).to_be_bytes());
                message.extend_from_slice(payload);

                Some(message)
            }
            Message::Piece { payload } => {
                let mut message = vec![];

                let payload_length = payload.len();

                message.extend_from_slice(&((payload_length + 1) as u32).to_be_bytes());
                message.extend_from_slice(&(7_u8).to_be_bytes());
                message.extend_from_slice(payload);

                Some(message)
            }
            Message::Request {
                piece_index,
                block_offset,
                block_length,
            } => {
                let mut message = vec![0, 0, 0, 13, 6];

                message.extend_from_slice(&piece_index.to_be_bytes());
                message.extend_from_slice(&block_offset.to_be_bytes());
                message.extend_from_slice(&block_length.to_be_bytes());

                Some(message)
            }
            Message::HandshakeResponse(info_hash, peer_id) => {
                let mut message = vec![];

                let protocol = &b"BitTorrent protocol".to_vec();
                let reserved = vec![0; 8];

                message.extend_from_slice(&(protocol.len() as u8).to_be_bytes());
                message.extend_from_slice(protocol);
                message.extend_from_slice(&reserved);
                message.extend_from_slice(info_hash);
                message.extend_from_slice(peer_id);

                Some(message)
            }
            _ => None,
        }
    }

    fn new(data: Vec<u8>) -> Self {
        let length = u32::from_be_bytes(data[0..4].try_into().expect("Incorrect message length"));

        match length {
            0 => Message::KeepAlive,
            _ => {
                let id = data[4];

                match id {
                    1 => Message::Unchoke,
                    2 => Message::Interested,
                    _ => {
                        let payload = data[5..(length + 4) as usize].to_vec();

                        match id {
                            4 => Message::Have { payload },
                            5 => Message::Bitfield { payload },
                            6 => {
                                let piece_index = u32::from_be_bytes(
                                    data[5..9]
                                        .try_into()
                                        .expect("Malformed request message: missing piece index"),
                                );

                                let block_offset = u32::from_be_bytes(
                                    data[9..13]
                                        .try_into()
                                        .expect("Malformed request message: missing block offset"),
                                );

                                let block_length = u32::from_be_bytes(
                                    data[13..17]
                                        .try_into()
                                        .expect("Malformed request message: missing block length"),
                                );

                                Message::Request {
                                    piece_index,
                                    block_length,
                                    block_offset,
                                }
                            }
                            7 => Message::Piece { payload },
                            _ => Message::Unrecognized,
                        }
                    }
                }
            }
        }
    }
}
