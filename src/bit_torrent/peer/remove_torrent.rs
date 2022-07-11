use std::{
    sync::{mpsc::Receiver, Arc, Mutex},
    thread::{self, JoinHandle},
};

use super::PeerState;

pub struct RemoveTorrent;

impl RemoveTorrent {
    pub fn wait_signal(
        pathname: String,
        remove_rx: Receiver<String>,
        state: Arc<Mutex<PeerState>>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            for recived in remove_rx {
                if recived == "End" {
                    log::info!(
                        "RemoveTorrent::wait_signal(): Removing torrent {}",
                        pathname
                    );
                    *state.lock().unwrap() = PeerState::Broken;
                    break;
                }
            }
        })
    }
}
