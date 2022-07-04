use super::{NetworkingError, Protocol};
use std::io::{Read, Write};

pub struct InterfaceProtocol<P: Protocol + Send + Sync + 'static> {
    protocol: P,
}

impl<P: Protocol + Send + Sync + 'static> InterfaceProtocol<P> {
    pub fn new(protocol: P) -> Self {
        Self { protocol }
    }

    pub fn get_protocol(&self) -> &P {
        &self.protocol
    }

    pub fn connect(&mut self, target_address: &str) -> Result<P::Stream, NetworkingError> {
        match P::connect(target_address) {
            Ok(stream) => Ok(stream),
            Err(_) => Err(NetworkingError::FailedToConnect),
        }
    }

    pub fn read_to_end(&mut self, to: &mut P::Stream) -> Result<Vec<u8>, NetworkingError> {
        let mut contents: Vec<u8> = Vec::new();

        match to.read_to_end(&mut contents) {
            Ok(_) => Ok(contents),
            Err(_) => Err(NetworkingError::FailedToRead),
        }
    }

    pub fn send<M: AsRef<[u8]>>(
        &mut self,
        to: &mut P::Stream,
        message: M,
    ) -> Result<(), NetworkingError> {
        match to.write_all(message.as_ref()) {
            Ok(_) => Ok(()),
            Err(_) => Err(NetworkingError::FailedToSend),
        }
    }
}
