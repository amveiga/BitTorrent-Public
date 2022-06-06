#![allow(dead_code)]

use super::Protocol;
use std::collections::HashMap;
use std::io::{Read, Write};

pub struct Client<P: Protocol + Send + Sync + 'static> {
    protocol: P,
    established_connections: HashMap<String, P::Stream>,
}

// un thread por cada conexión
// lock para algun registro de descargas
// algun lugar donde tengamos los join handles

impl<P: Protocol + Send + Sync + 'static> Client<P> {
    pub fn new(protocol: P) -> Self {
        Self {
            protocol,
            established_connections: HashMap::new(),
        }
    }

    pub fn connect(&mut self, target_address: &str) -> Result<(), String> {
        match P::connect(target_address) {
            Ok(stream) => {
                self.established_connections
                    .insert(target_address.to_string(), stream);

                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    // estas dos funciones cumplen mas de una cosa a la vez, podríamos separarlas en 3: send, read, get_stream
    pub fn send_and_read<M: AsRef<[u8]>>(
        &mut self,
        to: &str,
        message: M,
    ) -> Result<Vec<u8>, String> {
        let stream = self
            .established_connections
            .get_mut(to)
            .expect("Unexistent stream");

        stream
            .write_all(message.as_ref())
            .expect("Failed to write message");

        let mut contents: Vec<u8> = Vec::new();

        match stream.read_to_end(&mut contents) {
            Ok(_) => Ok(contents),
            Err(_) => Err(String::from("Failed to read incoming stream")),
        }
    }

    pub fn send_and_get_stream<M: AsRef<[u8]>>(&mut self, to: &str, message: M) -> &mut P::Stream {
        let stream = self
            .established_connections
            .get_mut(to)
            .expect("Unexistent stream");

        stream
            .write_all(message.as_ref())
            .expect("Failed to write message");

        stream
    }
}
