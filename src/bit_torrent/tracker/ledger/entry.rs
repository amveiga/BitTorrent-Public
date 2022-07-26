use super::{bencoder::Types, PeerList, PeerRecord};
use serde::Serialize;

#[derive(Debug, Default, Serialize, Clone)]
pub struct Entry {
    peer_list: PeerList,
}

impl Entry {
    pub fn new() -> Self {
        Self {
            peer_list: PeerList::default(),
        }
    }

    pub fn update(&mut self, peer: PeerRecord) {
        self.peer_list.update_entry(peer);
    }

    pub fn seeders(&self) -> usize {
        self.peer_list.seeders()
    }

    pub fn leechers(&self) -> usize {
        self.peer_list.leechers()
    }

    pub fn encode(&self) -> Types {
        self.peer_list.encode()
    }

    pub fn check(&mut self) {
        self.peer_list.check();
    }

    pub fn remove(&mut self, peer_id: [u8; 20]) {
        self.peer_list.remove(peer_id);
    }
}
