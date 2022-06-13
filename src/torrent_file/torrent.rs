#![allow(dead_code)]

use crate::bencoder::{decode::Decoder, Types};
use sha1::{Digest, Sha1};
use std::{collections::HashMap, fs::File, io::Read, vec};

pub struct Torrent {
    torrent_dict: HashMap<Vec<u8>, Types>,
    bytes: Vec<u8>,
}

impl Torrent {
    pub fn new_from_pathname(pathname: &str) -> Self {
        let torrent = File::open(pathname).expect("Failed to open torrent file");

        Self::new(torrent)
    }

    pub fn new(mut torrent_file: File) -> Self {
        let mut buffer = Vec::new();
        torrent_file
            .read_to_end(&mut buffer)
            .expect("Failed read torrent file");

        let torrent_dict = match Decoder::new_from_bytes(&buffer)
            .decode()
            .expect("Failed decoding torrent")
        {
            Types::Dictionary(dict) => dict,
            _ => HashMap::new(),
        };

        Self {
            torrent_dict,
            bytes: buffer,
        }
    }

    pub fn get_announce(&self) -> Option<Vec<u8>> {
        self.torrent_dict.get(&b"announce".to_vec())?.get_string()
    }

    pub fn get_files(&self) -> Option<&HashMap<Vec<u8>, Types>> {
        let info = self.torrent_dict.get(&b"info".to_vec())?.get_dictionary()?;
        info.get(&b"files".to_vec())?.get_dictionary()
    }

    pub fn get_length(&self) -> Option<i64> {
        let info = self.torrent_dict.get(&b"info".to_vec())?.get_dictionary()?;
        info.get(&b"length".to_vec())?.get_integrer()
    }

    pub fn get_name(&self) -> Option<Vec<u8>> {
        let info = self.torrent_dict.get(&b"info".to_vec())?.get_dictionary()?;
        info.get(&b"name".to_vec())?.get_string()
    }

    pub fn get_piece_length(&self) -> Option<i64> {
        let info = self.torrent_dict.get(&b"info".to_vec())?.get_dictionary()?;
        info.get(&b"piece length".to_vec())?.get_integrer()
    }

    pub fn get_pieces(&self) -> Option<Vec<Vec<u8>>> {
        let info = self.torrent_dict.get(&b"info".to_vec())?.get_dictionary()?;
        let hashes = info.get(&b"pieces".to_vec())?.get_string()?;
        let num_pieces: usize = hashes.len() / 20;
        let mut split_hashes: Vec<Vec<u8>> = vec![vec![0; 0]; num_pieces];
        for i in 0..num_pieces {
            split_hashes[i].extend_from_slice(&hashes[(i * 20)..((i + 1) * 20)])
        }

        Some(split_hashes)
    }

    pub fn get_info_hash(&self) -> [u8; 20] {
        let mut pos = 0;
        while pos + 7 <= self.bytes.len() {
            let slice = &self.bytes[pos..pos + 7];
            if slice.eq(b"4:infod") {
                break;
            };
            pos += 1;
        }
        let to_hash = &self.bytes[pos + 6..self.bytes.len() - 1];
        let mut hasher = Sha1::new();
        hasher.update(to_hash);

        hasher.finalize().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;
    #[test]
    fn test1_get_announce_from_torrent() {
        let filename =
            File::open("src/torrent_file/files_for_test/kubuntu-16.04.6-desktop-amd64.iso.torrent")
                .expect("Error in test-1:Could not open file");
        let torrent = Torrent::new(filename);
        let announce_result = torrent
            .get_announce()
            .expect("Error test 1 - Unable to get the announce ");
        let announce_expected = b"http://torrent.ubuntu.com:6969/announce".to_vec();
        assert_eq!(announce_result, announce_expected);
    }

    #[test]
    fn test2_get_lenght_from_torrent() {
        let filename =
            File::open("src/torrent_file/files_for_test/kubuntu-16.04.6-desktop-amd64.iso.torrent")
                .expect("Error in test-2:Could not open file");
        let torrent = Torrent::new(filename);
        let length_result = torrent
            .get_length()
            .expect("Error test 2 - Unable to get the length ");
        let length_expected = 1676083200;
        assert_eq!(length_result, length_expected);
    }

    #[test]
    fn test3_get_name_from_torrent() {
        let filename =
            File::open("src/torrent_file/files_for_test/kubuntu-16.04.6-desktop-amd64.iso.torrent")
                .expect("Error in test-3:Could not open file");
        let torrent = Torrent::new(filename);
        let name_result = torrent
            .get_name()
            .expect("Error test 3 - Unable to get the name ");
        let name_expected = b"kubuntu-16.04.6-desktop-amd64.iso".to_vec();
        assert_eq!(name_result, name_expected);
    }

    #[test]
    fn test4_get_piece_length_from_torrent() {
        let filename =
            File::open("src/torrent_file/files_for_test/kubuntu-16.04.6-desktop-amd64.iso.torrent")
                .expect("Error in test-4:Could not open file");
        let torrent = Torrent::new(filename);
        let piece_length_result = torrent
            .get_piece_length()
            .expect("Error test 4 - Unable to get the piece length ");
        let piece_length_expected = 524288;
        assert_eq!(piece_length_result, piece_length_expected);
    }
    #[test]
    fn test5_get_info_hash_from_torrent() {
        let filename =
            File::open("src/torrent_file/files_for_test/kubuntu-16.04.6-desktop-amd64.iso.torrent")
                .expect("Error in test-4:Could not open file");
        let torrent = Torrent::new(filename);
        let info_hash = torrent.get_info_hash();
        assert_eq!(info_hash, hex!("45b3d693cff285975f622acaeb75c5626acaff6f"));
    }
}
