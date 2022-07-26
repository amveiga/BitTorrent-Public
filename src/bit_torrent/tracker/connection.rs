pub use super::{
    bencoder::{Encoder, Types},
    constants::INTERVAL,
    networking::utils::get_available_port,
    Ledger, PeerRecord, Request, TrackerError, TrackerStatus,
};
use std::io::{Read, Write};
use std::net::TcpStream;
pub use std::sync::{Arc, Mutex};
use std::thread;
use std::{collections::HashMap, time::Instant};

pub struct Connection {
    stream: TcpStream,
    ledger: Arc<Mutex<Ledger>>,
    status: Arc<Mutex<TrackerStatus>>,
}

impl Connection {
    pub fn new(
        stream: TcpStream,
        ledger: Arc<Mutex<Ledger>>,
        status: Arc<Mutex<TrackerStatus>>,
    ) -> Self {
        Self {
            stream,
            ledger,
            status,
        }
    }

    pub fn activate(mut self) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let mut buffer = [0_u8; 1024];

            if self.stream.read(&mut buffer).is_ok() {
                match Request::new(
                    String::from_utf8_lossy(&buffer).to_string(),
                    self.stream.peer_addr().expect("Failed to get stream addr"),
                ) {
                    Ok(Request::Cors { allow, origin }) => {
                        if let Err(error) = self.manage_cross_origin(allow, origin) {
                            log::error!(
                                "Connection::activate() - {:?} manage_cross_origin(allow, origin)",
                                error
                            );
                        }
                    }
                    Ok(Request::Stats) => {
                        if let Err(error) = self.manage_stats() {
                            log::error!("Connection::activate() - {:?} from manage_stats()", error);
                        }
                    }
                    Ok(Request::Announce {
                        uploaded,
                        peer_id,
                        port,
                        info_hash,
                        left,
                        event,
                        ip,
                    }) => {
                        if self
                            .manage_announce(
                                info_hash,
                                PeerRecord {
                                    ip: ip.clone(),
                                    peer_id,
                                    port,
                                    uploaded,
                                    left,
                                    event,
                                    created_at: Instant::now(),
                                    updated_at: Instant::now(),
                                },
                            )
                            .is_ok()
                        {
                            log::info!("Connection::activate() - Successfully managed announce with peer {}:{}", ip, port);
                        } else {
                            log::error!("Connection::activate() - Failed to manage announce with peer {}:{}", ip, port);
                            if let Ok(mut ledger_guard) = self.ledger.lock() {
                                ledger_guard.remove_peer(info_hash, peer_id);
                            } else {
                                log::error!("Connection::activate() - Failed to lock ledger");
                                *self.status.lock().unwrap() = TrackerStatus::Break;
                            }
                        }
                    }
                    Err(error) => {
                        log::warn!("Connection::activate() - Failed to match request {}", error);
                        if let Err(err) = self.manage_error(error) {
                            log::warn!(
                                "Connection::activate() - Failed to send error to peer {:?}",
                                err
                            );
                        }
                    }
                }
            }
        })
    }

    pub fn manage_cross_origin(
        &mut self,
        allow: String,
        origin: String,
    ) -> Result<(), TrackerError> {
        self
      .stream
      .write_all(format!("HTTP/1.1 204 No Content\r\nConnection: keep-alive\r\nAccess-Control-Allow-Headers: *\r\nAccess-Control-Allow-Methods: {}\r\nAccess-Control-Allow-Origin: {}\r\nAccess-Control-Max-Age: 86400\r\n\r\n", allow, origin).as_bytes())
      .or(Err(TrackerError::FailedToSendResponse))?;

        Ok(())
    }

    pub fn manage_stats(&mut self) -> Result<(), TrackerError> {
        let response = serde_json::to_string(
            &*self
                .ledger
                .lock()
                .or(Err(TrackerError::UnableToLockLedger))?,
        )
        .or(Err(TrackerError::FailedToLockLedger))?;

        self
      .stream
      .write_all(
        format!(
          "HTTP/1.1 200 OK \r\nContent-Length:{}\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
          response.chars().count(),
          response
        )
        .as_bytes(),
      )
      .or(Err(TrackerError::FailedToSendResponse))?;

        Ok(())
    }

    pub fn manage_announce(
        &mut self,
        info_hash: [u8; 20],
        peer: PeerRecord,
    ) -> Result<(), TrackerError> {
        let mut ledger = self
            .ledger
            .lock()
            .or(Err(TrackerError::UnableToLockLedger))?;
        ledger.update(info_hash, peer);

        let mut response_information: HashMap<Vec<u8>, Types> = HashMap::new();

        let entry = ledger
            .get_entry(&info_hash)
            .ok_or(TrackerError::FailedToGetEntry)?;

        response_information.insert(b"complete".to_vec(), Types::Integer(entry.seeders() as i64));
        response_information.insert(
            b"incomplete".to_vec(),
            Types::Integer(entry.leechers() as i64),
        );

        response_information.insert(b"interval".to_vec(), Types::Integer(INTERVAL));
        response_information.insert(b"peers".to_vec(), entry.encode());

        let encoder = Encoder::new(Types::Dictionary(response_information));

        let response = String::from_utf8_lossy(
            &encoder
                .encode()
                .or(Err(TrackerError::FailedToEncodeResponse))?,
        )
        .to_string();

        self.stream
            .write_all(
                format!(
                    "HTTP/1.1 200 OK \r\nHost: 127.0.0.1:8080\r\nContent-Length:{}\r\nContent-Type: text/plain\r\n\r\n{}",
                    response.chars().count(),
                    response
                )
                .as_bytes(),
            ).or(Err(TrackerError::FailedToSendResponse))?;

        let _ = self.stream.flush();

        Ok(())
    }

    fn manage_error(&mut self, error: String) -> Result<(), TrackerError> {
        self.stream
            .write_all(format!("HTTP/1.1 400 Bad Request\r\nError: {}", error).as_bytes())
            .or(Err(TrackerError::FailedToSendResponse))?;

        Ok(())
    }
}
