use super::{bencoder::Types, constants::MAX_WAITTIME, PeerRecord};
use serde::Serialize;
use std::collections::{HashMap, LinkedList};

#[derive(Debug, Default, Serialize, Clone)]
pub struct PeerList {
    peers: Vec<PeerRecord>,
}

impl PeerList {
    pub fn seeders(&self) -> usize {
        self.peers
            .iter()
            .fold(0_usize, |accum: usize, item: &PeerRecord| {
                if item.event == "completed" {
                    return accum + 1;
                }
                accum
            })
    }

    pub fn leechers(&self) -> usize {
        self.peers
            .iter()
            .fold(0_usize, |accum: usize, item: &PeerRecord| {
                if item.event == "started" || item.event == "stopped" {
                    return accum + 1;
                }
                accum
            })
    }

    pub fn encode(&self) -> Types {
        let mut peers: LinkedList<Types> = LinkedList::new();

        self.peers.iter().for_each(|peer| {
            let mut peer_info: HashMap<Vec<u8>, Types> = HashMap::new();

            peer_info.insert(b"ip".to_vec(), Types::String(peer.ip.as_bytes().to_owned()));
            peer_info.insert(b"port".to_vec(), Types::Integer(peer.port as i64));

            peers.push_back(Types::Dictionary(peer_info))
        });

        Types::List(peers)
    }

    // pub fn update(&mut self, incoming_peers: Vec<PeerRecord>) {
    //   for peer in incoming_peers {
    //     if self
    //       .peers
    //       .iter()
    //       .any(|some_peer| some_peer.peer_id == peer.peer_id)
    //     {
    //       continue;
    //     } else {
    //       self.peers.push(peer);
    //     }
    //   }
    // }

    pub fn update_entry(&mut self, incoming_peer: PeerRecord) {
        if let Some(peer_to_update) = self
            .peers
            .iter_mut()
            .find(|some_peer| some_peer.peer_id == incoming_peer.peer_id)
        {
            peer_to_update.update(incoming_peer);
        } else {
            self.peers.push(incoming_peer);
        }
    }

    pub fn check(&mut self) {
        for (pos, peer_record) in self.peers.clone().iter().enumerate() {
            if peer_record.updated_at.elapsed().as_secs() > MAX_WAITTIME {
                self.peers.remove(pos);
            }
        }
    }

    pub fn remove(&mut self, peer_id: [u8; 20]) {
        if let Some(index) = self.peers.iter().position(|peer| peer.peer_id == peer_id) {
            self.peers.swap_remove(index);
        }
    }
}
