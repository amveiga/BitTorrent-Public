use super::{
    Bitfield, CommonInformation, PeerHandler, PeerList, Torrent, TorrentData, TrackerConnection,
};
use crate::file_system::File;
use gtk::glib::Sender;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Leecher {
    common_information: CommonInformation,
    have: Arc<Mutex<Bitfield>>,
    torrent: Torrent,
    peers: Arc<Mutex<PeerList>>,
}

impl Leecher {
    pub fn new(torrent_pathname: &str, sender: Arc<Mutex<Sender<TorrentData>>>) -> Self {
        let torrent = Torrent::new_from_pathname(torrent_pathname);

        let common_information = CommonInformation::new(&torrent, torrent_pathname, sender);

        let have = Arc::new(Mutex::new(Bitfield::new(common_information.total_pieces)));
        let peers = Arc::new(Mutex::new(PeerList::new()));

        Self {
            common_information,
            have,
            peers,
            torrent,
        }
    }

    pub fn activate(self) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let announce =
                String::from_utf8(self.torrent.get_announce().expect("Failed to get announce"))
                    .expect("Failed to parse announce");

            let tracker_connection = TrackerConnection::new(
                announce,
                Arc::clone(&self.have),
                Arc::clone(&self.peers),
                self.common_information.clone(),
            )
            .expect("Failed to create tracker connection");

            tracker_connection.activate(80);
            log::info!("Established connection with tracker");

            let peer_handler = PeerHandler::new(
                Arc::clone(&self.have),
                Arc::clone(&self.peers),
                self.common_information.clone(),
            )
            .expect("Failed to create peer handler");

            let download_thread = peer_handler.activate();

            download_thread.join().expect("Failed to join thread");

            let bitfield_guard = self.have.lock().unwrap();

            if bitfield_guard.is_complete() {
                File::join_pieces(
                    format!("./download_temp/{}", self.common_information.file_name),
                    format!("./{}", self.common_information.file_name),
                )
                .unwrap();
            }
        })
    }
}
