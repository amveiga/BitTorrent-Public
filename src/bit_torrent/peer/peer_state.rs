#[derive(Clone, Debug)]
pub enum PeerState {
    NoPieces(String),
    SomePieces(String),
    AllPieces(String),
    Broken,
}

impl PeerState {
    pub fn upgrade(&self, maybe_pathname: Option<String>) -> Self {
        match self {
            Self::NoPieces(pathname) => Self::SomePieces(pathname.to_string()),
            Self::SomePieces(pathname) => Self::AllPieces(if let Some(pathname) = maybe_pathname {
                pathname
            } else {
                pathname.to_string()
            }),
            Self::AllPieces(pathname) => Self::AllPieces(pathname.to_string()),
            Self::Broken => Self::Broken,
        }
    }
}
