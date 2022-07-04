use sha1::{Digest, Sha1};

use crate::{torrent_file::Torrent, utils::random_u64_as_bytes};

#[derive(Clone, Debug)]

pub struct CommonInformation {
    pub info_hash: Vec<u8>,
    pub peer_id: [u8; 20],
    pub pieces: Vec<Vec<u8>>,
    pub piece_length: usize,
    pub total_pieces: usize,
    pub file_name: String,
    pub file_length: u64,
}

impl CommonInformation {
    pub fn new(torrent: &Torrent) -> Self {
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
        }
    }
}
