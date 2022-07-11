use crate::frontend::torrents::TorrentData;

use super::{
    utils::split_u8, Bitfield, CommonInformation, Decoder, InterfaceProtocolHandler,
    NetworkingError, PeerList, PeerRecord, PeerState, Types, UrlEncoder,
};
use std::thread::{self};
use std::time::Instant;

use std::sync::{Arc, Mutex};

pub struct TrackerConnection {
    client: InterfaceProtocolHandler,
    bitfield: Arc<Mutex<Bitfield>>,
    peers: Arc<Mutex<PeerList>>,
    common_information: CommonInformation,
    tracker_address: String,
    sleep: u64,
    state: Arc<Mutex<PeerState>>,
}

impl TrackerConnection {
    pub fn new(
        announce: String,
        bitfield: Arc<Mutex<Bitfield>>,
        peers: Arc<Mutex<PeerList>>,
        common_information: CommonInformation,
        state: Arc<Mutex<PeerState>>,
    ) -> Result<Self, NetworkingError> {
        let (client, tracker_address) = InterfaceProtocolHandler::new(announce)?;

        Ok(Self {
            state,
            client,
            bitfield,
            peers,
            common_information,
            tracker_address,
            sleep: 2,
        })
    }

    pub fn activate(mut self, listening_port: u16, listening_ip: String) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let mut retries = 3;

            loop {
                if let PeerState::Broken = &*self.state.lock().unwrap() {
                    break;
                }

                let mut peers_guard = self.peers.lock().unwrap();
                if retries == 0 {
                    *self.state.lock().unwrap() = PeerState::Broken;
                    panic!("Tracker unavailable - No retries left");
                }
                let bitfield_guard = self.bitfield.lock().unwrap();

                let (downloaded, left) = bitfield_guard.status();

                drop(bitfield_guard);

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

                            let peers = PeerRecord::new_from_list(
                                list,
                                self.common_information.total_pieces,
                            );

                            peers_guard.update(peers);

                            peers_guard.remove(&listening_ip, listening_port as i64);

                            let bitfield_guard = self.bitfield.lock().unwrap();
                            retries = 3;
                            TorrentData::refresh(
                                &self.common_information,
                                &peers_guard,
                                &bitfield_guard,
                            );
                        }
                    } else {
                        retries -= 1;
                    }
                } else {
                    retries -= 1;
                }

                drop(peers_guard);

                let last_call = Instant::now();

                loop {
                    if let PeerState::Broken = &*self.state.lock().unwrap() {
                        break;
                    }

                    if last_call.elapsed().as_secs() >= self.sleep {
                        break;
                    }

                    thread::yield_now();
                }
            }
        })
    }
}
