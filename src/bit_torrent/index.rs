use super::{Leecher, TorrentData};

use gtk::glib::Sender;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

#[derive(Default)]

pub struct BitTorrent {
    as_leech: Vec<JoinHandle<()>>,
}

impl BitTorrent {
    pub fn new() -> Self {
        Self {
            as_leech: Vec::default(),
        }
    }

    pub fn new_leecher(&mut self, torrent_pathname: &str, sender: Arc<Mutex<Sender<TorrentData>>>) {
        let new_leecher = Leecher::new(torrent_pathname, sender);

        let handle = new_leecher.activate();

        self.as_leech.push(handle);
    }
}
