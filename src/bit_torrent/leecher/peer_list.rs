use super::Peer;

#[derive(Debug, Default)]
pub struct PeerList {
    peers: Vec<Peer>,
    in_use: usize,
}

impl PeerList {
    pub fn active(&self) -> usize {
        self.in_use
    }

    pub fn len(&self) -> usize {
        self.peers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.peers.is_empty()
    }

    pub fn new() -> Self {
        Self {
            peers: Vec::new(),
            in_use: 0,
        }
    }

    pub fn update(&mut self, incoming_peers: Vec<Peer>) {
        for peer in incoming_peers {
            if self.peers.iter().any(|some_peer| some_peer.ip == peer.ip) {
                continue;
            } else {
                self.peers.push(peer);
            }
        }
    }

    pub fn pop(&mut self) -> Option<Peer> {
        for peer in &mut self.peers {
            if !peer.in_use {
                peer.in_use = true;
                self.in_use += 1;
                return Some(peer.clone());
            }
        }

        None
    }

    pub fn remove(&mut self, ip: &str) {
        if let Some(index) = self.peers.iter().position(|peer| peer.ip == *ip) {
            if self.peers[index].in_use {
                self.in_use -= 1;
            }

            self.peers.swap_remove(index);
        }
    }
}
