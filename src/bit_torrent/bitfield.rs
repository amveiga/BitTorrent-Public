use std::convert::TryInto;

#[derive(Clone, Debug)]
pub struct Bitfield {
    total_pieces: usize,
    have: Vec<u8>,
    downloading: Vec<u8>,
}

impl Bitfield {
    pub fn is_complete(&self) -> bool {
        for i in 0..self.total_pieces {
            if !self.has(i) {
                return false;
            }
        }

        true
    }

    pub fn index_from_bytes(bytes: Vec<u8>) -> Result<usize, String> {
        let maybe_bytes: Result<[u8; 4], _> = bytes.try_into();

        match maybe_bytes {
            Ok(bytes) => Ok(i32::from_be_bytes(bytes) as usize),
            Err(_) => Err(String::from("Failed to parse index")),
        }
    }

    pub fn new(total_pieces: usize) -> Self {
        Self {
            total_pieces,
            have: vec![0; (total_pieces as f64 / 8.0).ceil() as usize],
            downloading: vec![0; (total_pieces as f64 / 8.0).ceil() as usize],
        }
    }

    pub fn new_from_vec(have: Vec<u8>, total_pieces: usize) -> Self {
        Self {
            total_pieces,
            have,
            downloading: vec![0; (total_pieces as f64 / 8.0).ceil() as usize],
        }
    }

    pub fn set(&mut self, index: usize) {
        let piece_index = index / 8;
        let piece_subindex = index % 8;

        self.have[piece_index] |= 128 >> piece_subindex;
        self.downloading[piece_index] &= !(128 >> piece_subindex);
    }

    pub fn set_downloading(&mut self, index: usize) {
        let piece_index = index / 8;
        let piece_subindex = index % 8;

        self.downloading[piece_index] |= 128 >> piece_subindex;
    }

    pub fn has(&self, index: usize) -> bool {
        let piece_index = index / 8;
        let piece_subindex = index % 8;

        let mask = 128 >> piece_subindex;

        self.have[piece_index] & mask != 0
    }

    pub fn is_downloading(&self, index: usize) -> bool {
        let piece_index = index / 8;
        let piece_subindex = index % 8;

        let mask = 128 >> piece_subindex;

        self.downloading[piece_index] & mask != 0
    }

    pub fn status(&self) -> (u64, u64) {
        let mut left = 0;

        for i in 0..self.total_pieces {
            if !self.has(i) {
                left += 1;
            }
        }

        (self.total_pieces as u64 - left, left)
    }

    pub fn first_needed_available_piece(&self, need: &Bitfield) -> Option<usize> {
        for i in 0..self.total_pieces {
            if self.has(i) && !need.has(i) && !need.is_downloading(i) {
                return Some(i);
            }
        }

        None
    }
}
