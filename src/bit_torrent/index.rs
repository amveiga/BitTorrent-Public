use crate::frontend::peers::PeersData;

use super::{Peer, TorrentData};

use gtk::glib::Sender;
use std::env;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

#[derive(Default)]

pub struct BitTorrent {
    processes: Vec<JoinHandle<()>>,
}

impl BitTorrent {
    pub fn new() -> Self {
        Self {
            processes: Vec::default(),
        }
    }

    pub fn new_process(
        &mut self,
        torrent_pathname: &str,
        sender_torrent: Arc<Mutex<Sender<TorrentData>>>,
        sender_peers: Arc<Mutex<Sender<PeersData>>>,
        remove_rx: Receiver<String>,
    ) {
        let new_process = Peer::new(
            torrent_pathname,
            env::var("TEMP_PATH")
                .unwrap_or_else(|_| "".to_string())
                .as_str(),
            env::var("DOWNLOAD_PATH")
                .unwrap_or_else(|_| "".to_string())
                .as_str(),
            sender_torrent,
            sender_peers,
        );

        let handle = new_process.activate(remove_rx);

        self.processes.push(handle);
    }

    pub fn wait_for_thereads(&mut self) {
        while let Some(process) = self.processes.pop() {
            process.join().unwrap();
        }
    }
}
