use super::{Leecher, Process, Seeder};

#[derive(Default)]

pub struct BitTorrent {
    as_leech: Vec<Process<Leecher>>,
    as_seeder: Vec<Process<Seeder>>,
}

impl BitTorrent {
    pub fn new() -> Self {
        Self {
            as_leech: Vec::default(),
            as_seeder: Vec::default(),
        }
    }

    pub fn new_seeder(&mut self, torrent_pathname: &str) {
        self.as_seeder.push(Process::new(torrent_pathname));
    }

    pub fn new_leecher(&mut self, torrent_pathname: &str) {
        self.as_leech.push(Process::new(torrent_pathname));
    }
}
