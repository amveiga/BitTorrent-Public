use super::Job;
use std::convert::AsRef;
use std::io::{Read, Write};
use std::net::TcpStream;

pub trait Protocol {
    type Stream: Read + Write;

    fn connect(target_address: &str) -> Result<Self::Stream, String>;

    fn handle_incoming(self, stream: TcpStream) -> Job;

    fn handle_request<R: AsRef<[u8]>>(&self, request: R, stream: TcpStream) -> Job;
}
