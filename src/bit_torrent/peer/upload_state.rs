#[derive(Clone, Debug)]
pub enum State {
    UnknownPeer,
    AwaitingResponse,
    Uploading,
    Useless,
}
