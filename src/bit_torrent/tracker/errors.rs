#[derive(Debug)]

pub enum TrackerError {
    NoPortAvailable,
    FailedToBindStream,
    FailedToSendResponse,
    FailedToEncodeResponse,
    FailedToGetEntry,
    FailedToLockLedger,
    UnableToGetPeerAddr,
    UnableToLockLedger,
}
