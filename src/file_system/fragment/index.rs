use std::convert::From;

pub struct Fragment {
    content: Vec<u8>,
}

impl Fragment {
    pub fn get_content(&self) -> &Vec<u8> {
        &self.content
    }
}

impl From<Vec<u8>> for Fragment {
    fn from(bytes: Vec<u8>) -> Self {
        Self { content: bytes }
    }
}
