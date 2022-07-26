use super::{HTTPSTracker, HTTPTracker, Handshake, InterfaceProtocol, NetworkingError, Protocol};

pub enum InterfaceProtocolHandler {
    Http(
        InterfaceProtocol<HTTPTracker>,
        <HTTPTracker as Protocol>::Stream,
    ),
    Https(
        InterfaceProtocol<HTTPSTracker>,
        <HTTPSTracker as Protocol>::Stream,
    ),
}

impl InterfaceProtocolHandler {
    pub fn new(tracker_address: String) -> Result<(Self, String), NetworkingError> {
        let address_split = tracker_address.split("://").collect::<Vec<&str>>();

        let protocol = address_split[0];
        let address = address_split[1].to_string();

        let address_split: Vec<&str> = address.split('/').collect();

        let base = address_split[0].to_string();
        let endpoint = address_split[1].to_string();

        match protocol {
            "http" => {
                let mut client = InterfaceProtocol::new(HTTPTracker::new(Some(endpoint)));

                if let Ok(stream) = client.connect(&base) {
                    return Ok((Self::Http(client, stream), base));
                }
                Err(NetworkingError::FailedToConnect)
            }
            "https" => {
                let base = format!("{}:443", base);

                let mut client = InterfaceProtocol::new(HTTPSTracker::new(Some(endpoint)));

                if let Ok(stream) = client.connect(&base) {
                    return Ok((Self::Https(client, stream), base));
                }
                Err(NetworkingError::FailedToConnect)
            }
            _ => Err(NetworkingError::FailedToConnect),
        }
    }

    pub fn send(&mut self, request: String) -> Result<(), NetworkingError> {
        match self {
            Self::Http(ref mut client, ref mut stream) => client.send(stream, request),
            Self::Https(ref mut client, ref mut stream) => client.send(stream, request),
        }
    }

    pub fn read_to_end(&mut self) -> Result<Vec<u8>, NetworkingError> {
        match self {
            Self::Http(ref mut client, ref mut stream) => client.read_to_end(stream),
            Self::Https(ref mut client, ref mut stream) => client.read_to_end(stream),
        }
    }

    pub fn format_handshake_message(&self, handshake_params: Handshake) -> String {
        match self {
            Self::Http(ref client, _) => client
                .get_protocol()
                .format_handshake_message(handshake_params),
            Self::Https(ref client, _) => client
                .get_protocol()
                .format_handshake_message(handshake_params),
        }
    }
}
