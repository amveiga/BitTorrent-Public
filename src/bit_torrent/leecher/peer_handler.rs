use super::{Bitfield, CommonInformation, Connections, NetworkingError, PeerConnection, PeerList};
use std::thread::{self, sleep, JoinHandle};

use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct PeerHandler {
    bitfield: Arc<Mutex<Bitfield>>,
    peers: Arc<Mutex<PeerList>>,
    common_information: CommonInformation,
    connections: Arc<Mutex<Connections>>,
}

impl PeerHandler {
    pub fn new(
        bitfield: Arc<Mutex<Bitfield>>,
        peers: Arc<Mutex<PeerList>>,
        common_information: CommonInformation,
    ) -> Result<Self, NetworkingError> {
        Ok(Self {
            bitfield,
            peers,
            common_information,
            connections: Arc::new(Mutex::new(Connections::new())),
        })
    }

    pub fn activate(self) -> JoinHandle<()> {
        thread::spawn(move || loop {
            let bitfield_guard = self.bitfield.lock().unwrap();
            if bitfield_guard.is_complete() {
                log::info!("PeerHandler::activate() - File is completely downloaded");
                break;
            }
            drop(bitfield_guard);

            let mut peers_guard = self.peers.lock().unwrap();
            if peers_guard.active() == 40 {
                drop(peers_guard);
                thread::yield_now();
                continue;
            }

            let maybe_peer = peers_guard.pop();

            drop(peers_guard);

            match maybe_peer {
                Some(peer_in_use) => {
                    let mut connections_guard = self.connections.lock().unwrap();

                    let new_connection = PeerConnection::activate(
                        Arc::clone(&self.bitfield),
                        Arc::clone(&self.peers),
                        self.common_information.clone(),
                        peer_in_use.clone(),
                        Arc::clone(&self.connections),
                    );

                    connections_guard.add(peer_in_use.ip, new_connection);

                    drop(connections_guard);
                }
                None => {
                    log::trace!("PeerHandler::activate() - No peers available");

                    sleep(Duration::from_secs(1));
                }
            };
        })
    }
}
