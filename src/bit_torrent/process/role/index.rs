pub trait Role {
    fn new(torrent_pathname: &str) -> Self;
    fn activate(&mut self);
    fn deactivate(&mut self);
}
