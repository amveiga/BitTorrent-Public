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
        thread::spawn(move || loop {
            if let PeerState::Broken = &*state.lock().unwrap() {
                break;
            }

            let maybe_recv = remove_rx.try_recv();
            if let Ok(recv) = maybe_recv {
                if recv == "End" {
                    log::info!(
                        "RemoveTorrent::wait_signal(): Removing torrent {}",
                        pathname
                    );
                    *state.lock().unwrap() = PeerState::Broken;
                    break;
                }
            }

            thread::yield_now();
        })
    }
}
