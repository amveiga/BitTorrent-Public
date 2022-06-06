use std::io::Read;
use std::net::TcpStream;

#[derive(Debug)]

pub enum Message {
    Unrecognized,
    KeepAlive,
    Interested,
    Unchoke,
    Request { payload: Vec<u8> },
    Bitfield { payload: Vec<u8> },
    Piece { payload: Vec<u8> },
}

impl Message {
    pub fn validate_stream_handshake(
        stream: &mut TcpStream,
        handshake_len: usize,
        info_hash: &[u8; 20],
    ) -> Result<bool, String> {
        let mut response = vec![0_u8; handshake_len];

        match stream.read_exact(&mut response) {
            Ok(_) => {
                if response.len() == handshake_len
                    && response[handshake_len - 40..handshake_len - 20].starts_with(info_hash)
                // falta checkear que es el expected peer_id
                {
                    return Ok(true);
                }
                Ok(false)
            }
            Err(_) => Err(String::from("Failed to read handshake from stream")),
        }
    }

    pub fn read_message_from_stream(stream: &mut TcpStream) -> Result<Message, String> {
        let mut header_buffer = [0_u8; 5];

        match stream.read_exact(&mut header_buffer) {
            Ok(_) => {
                let mut message = vec![];
                message.extend_from_slice(&header_buffer);

                let payload_length = Message::read_length_from_header(&header_buffer);

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
            Err(_) => Err(String::from("Failed to read header from stream")),
        }
    }

    pub fn read_length_from_header(header: &[u8; 5]) -> u32 {
        u32::from_be_bytes(header[0..4].try_into().expect("Incorrect message length")) - 1
    }

    pub fn parse(&self) -> Option<Vec<u8>> {
        match self {
            Message::Interested => Some(vec![0, 0, 0, 1, 2]),
            Message::Unrecognized => None,
            Message::Request { payload } => {
                let mut header = vec![0, 0, 0, 13, 6];
                header.extend_from_slice(payload);

                Some(header)
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
                            5 => Message::Bitfield { payload },
                            6 => Message::Request { payload },
                            7 => Message::Piece { payload },
                            _ => Message::Unrecognized,
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1_interested_parse() {
        let interesetd = Message::Interested
            .parse()
            .expect("test_interested_parse - Failed to parse interested");
        let expected = vec![0, 0, 0, 1, 2];

        assert_eq!(interesetd, expected);
    }
}
