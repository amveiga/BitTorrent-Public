use std::sync::{Arc, Mutex};

use sha1::{Digest, Sha1};

use crate::{
    frontend::{peers::PeersData, torrents::TorrentData},
    torrent_file::Torrent,
    utils::random_u64_as_bytes,
};

#[derive(Clone, Debug)]

pub struct CommonInformation {
    pub info_hash: Vec<u8>,
    pub peer_id: [u8; 20],
    pub pieces: Vec<Vec<u8>>,
    pub piece_length: usize,
    pub total_pieces: usize,
    pub file_name: String,
    pub file_length: u64,
    pub torrent_pathname: String,
    pub tx_torrent: Arc<Mutex<gtk::glib::Sender<TorrentData>>>,
    pub tx_peers: Arc<Mutex<gtk::glib::Sender<PeersData>>>,
    pub temp_directory: String,
    pub download_directory: String,
}

impl CommonInformation {
    pub fn new(
        torrent: &Torrent,
        torrent_pathname: &str,
        temp_directory: &str,
        download_directory: &str,
        tx_torrent: Arc<Mutex<gtk::glib::Sender<TorrentData>>>,
        tx_peers: Arc<Mutex<gtk::glib::Sender<PeersData>>>,
    ) -> Self {
        let pieces = torrent
            .get_pieces()
            .expect("Corrupted torrent, no pieces attribute");

        let file_name = String::from_utf8_lossy(
            &torrent
                .get_name()
                .expect("Corrupted torrent, no name attribute"),
        )
        .to_string();

        let piece_length = torrent
            .get_piece_length()
            .expect("Corrupted torrent, no piece length") as usize;

        let info_hash = torrent.get_info_hash().to_vec();

        let mut hasher = Sha1::new();
        hasher.update(random_u64_as_bytes());

        let peer_id: [u8; 20] = hasher.finalize().into();

        let file_length = torrent.get_length().expect("Corrupted torrent, no length") as u64;

        Self {
            piece_length,
            peer_id,
            total_pieces: pieces.len(),
            pieces,
            info_hash,
            file_name,
            file_length,
            torrent_pathname: torrent_pathname.to_string(),
            tx_torrent,
            tx_peers,
            temp_directory: temp_directory.to_string(),
            download_directory: download_directory.to_string(),
        }
    }
}
