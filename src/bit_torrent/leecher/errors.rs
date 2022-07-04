#[derive(Debug)]
pub enum Error {
    InvalidPiece,
    FailedToSavePiece,
    NoNewPiecesFromPeer,
    FailedToUnchoke,
}
