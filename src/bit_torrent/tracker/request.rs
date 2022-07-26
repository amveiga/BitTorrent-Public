use super::{networking::HTTPTracker, urlencoder::encode::UrlEncoder};
use std::net::SocketAddr;

#[derive(Debug)]
pub enum Request {
    Announce {
        peer_id: [u8; 20],
        info_hash: [u8; 20],
        port: u16,
        uploaded: u64,
        left: u64,
        event: String,
        ip: String,
    },
    Stats,
    Cors {
        origin: String,
        allow: String,
    },
}

impl Request {
    pub fn new(full_request: String, addr: SocketAddr) -> Result<Self, String> {
        let request_split: Vec<&str> = full_request.split("\r\n").collect();

        let request = request_split[0].to_string();
        let headers: Vec<&str> = request_split[1..request_split.len() - 1].to_vec();

        let request_split: Vec<&str> = request.split(' ').collect();

        let method = <&str>::clone(&request_split[0]);
        let query_url = request_split[1].to_string();

        let request_split: Vec<&str> = query_url.split('?').collect();

        let path = <&str>::clone(&request_split[0]);

        let query = if request_split.len() > 1 {
            <&str>::clone(&request_split[1])
        } else {
            ""
        };
        match method {
            "OPTIONS" => {
                let mut allow: Option<String> = None;
                let mut origin: Option<String> = None;

                for header in headers {
                    if (allow.is_some() && origin.is_some()) || header.is_empty() {
                        break;
                    }

                    let split: Vec<&str> = header.split(": ").collect();
                    let key = split[0].to_owned();
                    let value = split[1].to_owned();

                    if key == "Access-Control-Request-Method" {
                        allow = Some(value);
                    } else if key == "Origin" {
                        origin = Some(value);
                    }
                }

                if let (Some(allow), Some(origin)) = (allow, origin) {
                    Ok(Request::Cors { allow, origin })
                } else {
                    Err("Invalid request".to_string())
                }
            }

            "GET" => match path {
                "/stats" => Ok(Self::Stats),
                "/announce" => {
                    let query_map = HTTPTracker::query_to_hashmap(query);

                    let port: u16 = query_map
                        .get("port")
                        .ok_or_else(|| String::from("Missing required attribute: Port"))?
                        .clone()
                        .parse::<u16>()
                        .map_err(|_| String::from("Failed to parse: Port"))?;

                    let uploaded: u64 = query_map
                        .get("uploaded")
                        .ok_or_else(|| String::from("Missing required attribute: Uploaded"))?
                        .clone()
                        .parse::<u64>()
                        .map_err(|_| String::from("Failed to parse: Uploaded"))?;

                    let left: u64 = query_map
                        .get("left")
                        .ok_or_else(|| String::from("Missing required attribute: Left"))?
                        .clone()
                        .parse::<u64>()
                        .map_err(|_| String::from("Failed to parse: Left"))?;

                    let event: String = query_map
                        .get("event")
                        .ok_or_else(|| String::from("Missing required attribute: Event"))?
                        .clone();

                    let info_hash = query_map
                        .get("info_hash")
                        .ok_or_else(|| String::from("Missing required attribute: Info hash"))?
                        .clone();

                    let info_hash = UrlEncoder::decode_binary_data(info_hash)
                        .map_err(|_| String::from("Failed to parse: Info hash"))?;

                    let peer_id = query_map
                        .get("peer_id")
                        .ok_or_else(|| String::from("Missing required attribute: Peer id"))?
                        .clone();

                    let peer_id = if peer_id.contains('%') {
                        UrlEncoder::decode_binary_data(peer_id)
                            .map_err(|_| String::from("Failed to parse: Peer id"))?
                    } else {
                        peer_id.as_bytes().to_vec()
                    };

                    Ok(Self::Announce {
                        ip: addr.ip().to_string(),
                        port,
                        info_hash: info_hash
                            .try_into()
                            .map_err(|_| String::from("Failed to parse: Info hash into shape"))?,
                        peer_id: peer_id
                            .try_into()
                            .map_err(|_| String::from("Failed to parse: Peer id into shape"))?,
                        uploaded,
                        left,
                        event,
                    })
                }
                _ => Err(String::from("Invalid EP")),
            },
            _ => Err(String::from("Invalid method")),
        }
    }
}
