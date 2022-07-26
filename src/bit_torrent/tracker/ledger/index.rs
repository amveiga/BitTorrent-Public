use super::{utils::u8_to_hexa, Entry, PeerRecord};
use serde::{ser::SerializeMap, Serialize, Serializer};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Ledger {
    entries: HashMap<[u8; 20], Entry>,
}

impl Serialize for Ledger {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.entries.len()))?;

        for (k, v) in self.entries.iter() {
            map.serialize_entry(&u8_to_hexa(k).to_string(), v)?;
        }
        map.end()
    }
}

impl Ledger {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn get_entry(&self, key: &[u8; 20]) -> Option<&Entry> {
        self.entries.get(key)
    }

    // pub fn use_serialize(&self) -> String {
    //   self.serialize().unwrap()
    // }

    pub fn new_entry(&mut self, key: [u8; 20]) {
        self.entries.insert(key, Entry::new());
    }

    pub fn update(&mut self, info_hash: [u8; 20], peer: PeerRecord) {
        if !self.entries.contains_key(&info_hash) {
            self.new_entry(info_hash);
        };

        if let Some(entry) = self.entries.get_mut(&info_hash) {
            if peer.event == "stopped" {
                entry.remove(peer.peer_id);
            } else {
                entry.update(PeerRecord::new(
                    peer.ip,
                    peer.peer_id,
                    peer.port,
                    peer.uploaded,
                    peer.left,
                    peer.event,
                ))
            }
        }
    }

    pub fn remove_peer(&mut self, info_hash: [u8; 20], peer_id: [u8; 20]) {
        if let Some(entry) = self.entries.get_mut(&info_hash) {
            entry.remove(peer_id);
        }
    }

    pub fn check(&mut self) {
        for (_, entry) in self.entries.iter_mut() {
            entry.check();
        }
    }
}
