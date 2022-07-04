use super::{
    utils::split_u8, Bitfield, ClientHandler, CommonInformation, Decoder, NetworkingError, Peer,
    PeerList, Types, UrlEncoder,
};
use std::thread::{self};

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

pub struct TrackerConnection {
    client: ClientHandler,
    bitfield: Arc<Mutex<Bitfield>>,
    peers: Arc<Mutex<PeerList>>,
    common_information: CommonInformation,
    tracker_address: String,
    sleep: u64,
}

impl TrackerConnection {
    pub fn new(
        announce: String,
        bitfield: Arc<Mutex<Bitfield>>,
        peers: Arc<Mutex<PeerList>>,
        common_information: CommonInformation,
    ) -> Result<Self, NetworkingError> {
        let (client, tracker_address) = ClientHandler::new(announce)?;

        Ok(Self {
            client,
            bitfield,
            peers,
            common_information,
            tracker_address,
            sleep: 2,
        })
    }

    pub fn activate(mut self, listening_port: u16) {
        thread::spawn(move || {
            let mut retries = 3;

            loop {
                let mut peers_guard = self.peers.lock().unwrap();
                if retries == 0 {
                    log::error!("TrackerConnection::activate() - No retries left");
                    panic!("Tracker unavailable - No retries left");
                }
                log::debug!("TrackerConnection::activate() - trying to obtain bitfield lock");
                let bitfield_guard = self.bitfield.lock().unwrap();
                log::debug!("TrackerConnection::activate() - bitfield lock obtained");
                let (downloaded, left) = bitfield_guard.status();

                drop(bitfield_guard);
                log::debug!("TrackerConnection::activate() - bitfield lock dropped");

                let request = self.client.format_handshake_message(
                    &self.tracker_address,
                    UrlEncoder::encode_binary_data(self.common_information.peer_id.as_ref()),
                    UrlEncoder::encode_binary_data(&self.common_information.info_hash),
                    listening_port,
                    left,
                    downloaded,
                );

                let response = if let Err(error) = self.client.send(request) {
                    Err(error)
                } else {
                    self.client.read_to_end()
                };

                if let Ok(response) = response {
                    // TODO: Hay que ver cuando nos responde algo que no esperamos porque rompe muy cada tanto
                    let slice = split_u8(response, b"\r\n\r\n");

                    if let Ok(Types::Dictionary(dict)) = Decoder::new_from_bytes(&slice).decode() {
                        let peers = dict
                            .get(&b"peers".to_vec())
                            .expect("Failed to parse peers from tracker response");

                        let sleep = dict
                            .get(&b"interval".to_vec())
                            .expect("Failed to get interval from tracker response")
                            .get_integrer()
                            .expect("Failed to parse interval from tracker response");

                        self.sleep = sleep as u64;

                        if let Types::List(list) = peers {
                            log::debug!(
                                "TrackerConnection::activate() - trying to obtain PeerList lock"
                            );
                            log::debug!("TrackerConnection::activate() - PeerList lock obtained");

                            let peers =
                                Peer::new_from_list(list, self.common_information.total_pieces);
                            peers_guard.update(peers);

                            retries = 3;
                        }
                    } else {
                        retries -= 1;
                    }
                } else {
                    retries -= 1;
                }

                drop(peers_guard);
                log::debug!("TrackerConnection::activate() - PeerList lock dropped");

                thread::sleep(Duration::from_secs(self.sleep));
            }
        });
    }
}
