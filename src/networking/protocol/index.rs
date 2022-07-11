use std::io::{Read, Write};

pub trait Protocol {
    type Stream: Read + Write;

    fn connect(target_address: &str) -> Result<Self::Stream, String>;
}
