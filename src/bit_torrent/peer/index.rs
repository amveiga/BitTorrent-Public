use crate::bit_torrent::peer::RemoveTorrent;
use crate::frontend::peers::PeersData;

use super::{
    Bitfield, CommonInformation, PeerHandler, PeerList, PeerState, ServerHandler, Torrent,
    TorrentData, TrackerConnection,
};
use gtk::glib::Sender;
use std::sync::mpsc::Receiver;
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

impl Peer {
    pub fn new(
        torrent_pathname: &str,
        temp_directory: &str,
        download_directory: &str,
        sender_torrent: Arc<Mutex<Sender<TorrentData>>>,
        sender_peers: Arc<Mutex<Sender<PeersData>>>,
    ) -> Self {
        let torrent = Torrent::new_from_pathname(torrent_pathname);

        let common_information = CommonInformation::new(
            &torrent,
            torrent_pathname,
            temp_directory,
            download_directory,
            sender_torrent,
            sender_peers,
        );

        let have = Arc::new(Mutex::new(Bitfield::new(common_information.total_pieces)));
        let peers = Arc::new(Mutex::new(PeerList::new()));

        Self {
            state: Arc::new(Mutex::new(PeerState::NoPieces(format!(
                "{}/{}",
                temp_directory, common_information.file_name
            )))),
            common_information,
            have,
            peers,
            torrent,
            handlers: Vec::new(),
        }
    }

    pub fn activate(mut self, remove_rx: Receiver<String>) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            self.handlers.push(RemoveTorrent::wait_signal(
                self.common_information.torrent_pathname.clone(),
                remove_rx,
                Arc::clone(&self.state),
            ));

            let announce =
                String::from_utf8(self.torrent.get_announce().expect("Failed to get announce"))
                    .expect("Failed to parse announce");

            let server_handler = ServerHandler::new(
                Arc::clone(&self.have),
                self.common_information.clone(),
                Arc::clone(&self.state),
            )
            .expect("Failed to create server handler");

            let tracker_connection = TrackerConnection::new(
                announce,
                Arc::clone(&self.have),
                Arc::clone(&self.peers),
                self.common_information.clone(),
                Arc::clone(&self.state),
            )
            .expect("Failed to create tracker connection");

            self.handlers.push(
                tracker_connection.activate(server_handler.get_port(), server_handler.get_ip()),
            );

            self.handlers.push(server_handler.activate());

            log::info!("Established connection with tracker");

            let peer_handler = PeerHandler::new(
                Arc::clone(&self.have),
                Arc::clone(&self.peers),
                self.common_information.clone(),
                Arc::clone(&self.state),
            )
            .expect("Failed to create peer handler");

            self.handlers.push(peer_handler.activate());

            for handler in self.handlers {
                handler.join().expect("Failed tu join thread handler");
            }

            log::info!("Peer::activate(): All threads joined");
        })
    }
}
