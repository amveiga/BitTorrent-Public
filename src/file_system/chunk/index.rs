use std::convert::From;

pub struct Chunk {
    content: Vec<u8>,
}

impl Chunk {
    pub fn get_content(&self) -> &Vec<u8> {
        &self.content
    }
}

impl From<Vec<u8>> for Chunk {
    fn from(bytes: Vec<u8>) -> Self {
        Self { content: bytes }
    }
}
