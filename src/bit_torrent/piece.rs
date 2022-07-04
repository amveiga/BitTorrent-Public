use super::File;
use sha1::{Digest, Sha1};
use std::io::Error;

pub struct Piece {
    id: usize,
    data: Vec<u8>,
    block_index: usize,
    block_offset: usize,
    total_blocks: usize,
    block_size: usize,
    last_block_size: usize,
    piece_size: usize,
}

impl Piece {
    pub fn new(id: usize, piece_size: usize, block_size: usize) -> Self {
        let total_blocks = (piece_size as f64 / block_size as f64).ceil() as usize;

        let last_block_size = (piece_size - block_size * (total_blocks - 1)) as usize;

        Self {
            piece_size,
            last_block_size,
            block_size,
            id,
            data: Vec::new(),
            block_index: 0,
            total_blocks,
            block_offset: 0,
        }
    }

    pub fn add_block(&mut self, mut new_block: Vec<u8>) {
        self.block_index += 1;

        if self.block_index == self.total_blocks - 1 {
            self.block_offset += self.last_block_size;
        } else {
            self.block_offset += self.block_size;
        }

        self.data.append(&mut new_block);
    }

    pub fn get_next_block_attributes(&self) -> (usize, usize, usize) {
        if self.total_blocks == 1 || self.block_index == self.total_blocks - 2 {
            (self.id, self.block_offset, self.last_block_size)
        } else {
            (self.id, self.block_offset, self.block_size)
        }
    }

    pub fn is_complete(&self) -> bool {
        self.data.len() == self.piece_size
    }

    pub fn verify(&self, hash: Vec<u8>) -> bool {
        let mut encoder = Sha1::new();
        encoder.update(self.data.clone());
        let piece_hash: [u8; 20] = encoder.finalize().into();

        piece_hash.to_vec() == hash
    }

    pub fn save(&self, filename: &str) -> Result<File, Error> {
        File::new_file_from_piece(&self.data, format!("{}.piece{}", filename, self.id))
    }
}
