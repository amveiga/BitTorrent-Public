#[derive(Clone, Debug)]
pub enum State {
    UnknownToPeer,
    ProcessingHandshakeResponse,
    Choked,
    Downloading,
    FileDownloaded,
    Useless(bool),
}
