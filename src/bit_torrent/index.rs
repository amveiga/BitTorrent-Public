use super::Leecher;

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

    pub fn new_leecher(&mut self, torrent_pathname: &str) {
        let new_leecher = Leecher::new(torrent_pathname);

        let handle = new_leecher.activate();

        self.as_leech.push(handle);
    }
}
