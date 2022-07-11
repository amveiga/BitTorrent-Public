use super::{
    file_system::File, Bitfield, CommonInformation, NetworkingError, PeerConnection, PeerList,
    PeerState,
};
use std::thread::{self, JoinHandle};

use std::sync::{Arc, Mutex};

pub struct PeerHandler {
    bitfield: Arc<Mutex<Bitfield>>,
    peers: Arc<Mutex<PeerList>>,
    common_information: CommonInformation,
    state: Arc<Mutex<PeerState>>,
}

impl PeerHandler {
    pub fn new(
        bitfield: Arc<Mutex<Bitfield>>,
        peers: Arc<Mutex<PeerList>>,
        common_information: CommonInformation,
        state: Arc<Mutex<PeerState>>,
    ) -> Result<Self, NetworkingError> {
        Ok(Self {
            state,
            bitfield,
            peers,
            common_information,
        })
    }

    pub fn activate(self) -> JoinHandle<()> {
        thread::spawn(move || {
            let mut connections: Vec<JoinHandle<()>> = vec![];

            loop {
                if let PeerState::Broken = &*self.state.lock().unwrap() {
                    break;
                }

                let bitfield_guard = self.bitfield.lock().unwrap();

                if !bitfield_guard.is_null() {
                    let mut state_guard = self.state.lock().unwrap();
                    if let PeerState::NoPieces(_) = &*state_guard {
                        *state_guard = state_guard.upgrade(None);
                    }
                }

                if bitfield_guard.is_complete() {
                    if bitfield_guard.is_complete() {
                        File::join_pieces(
                            format!(
                                "{}/{}",
                                self.common_information.temp_directory,
                                self.common_information.file_name
                            ),
                            format!(
                                "{}/{}",
                                self.common_information.download_directory,
                                self.common_information.file_name
                            ),
                        )
                        .unwrap();
                    }

                    let mut state_guard = self.state.lock().unwrap();
                    if let PeerState::SomePieces(_) = &*state_guard {
                        *state_guard = state_guard.upgrade(Some(format!(
                            "{}/{}",
                            self.common_information.download_directory,
                            self.common_information.file_name
                        )));
                    }

                    break;
                }
                drop(bitfield_guard);

                let peers_guard = self.peers.lock();
                match peers_guard {
                    Ok(mut peers_guard) => {
                        if peers_guard.active() == 30 {
                            drop(peers_guard);
                            thread::yield_now();
                            continue;
                        }

                        let maybe_peer = peers_guard.pop();

                        drop(peers_guard);
                        if let Some(peer_in_use) = maybe_peer {
                            connections.push(PeerConnection::activate(
                                Arc::clone(&self.bitfield),
                                Arc::clone(&self.peers),
                                self.common_information.clone(),
                                peer_in_use.clone(),
                                Arc::clone(&self.state),
                            ));
                        }
                    }
                    Err(_) => {
                        break;
                    }
                }
            }

            for connection in connections {
                connection
                    .join()
                    .expect("Failed to join peer connection TUKI");
            }
        })
    }
}
