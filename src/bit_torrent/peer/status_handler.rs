use super::{
  Bitfield, CommonInformation, PeerHandler, PeerList, PeerState, ServerHandler, Torrent,
  TorrentData, TrackerConnection,
};
use gtk::glib::Sender;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Peer {
  common_information: CommonInformation,
  have: Arc<Mutex<Bitfield>>,
  torrent: Torrent,
  peers: Arc<Mutex<PeerList>>,
  state: Arc<Mutex<PeerState>>,
  handlers: Vec<thread::JoinHandle<()>>,
}

pub struct StatusHandler {
  rx
}