#[derive(Debug)]

pub enum NetworkingError {
    InvalidTrackerAddress,
    FailedToSend,
    FailedToRead,
    FailedToConnect,
    FailedPeerConnection,
}
