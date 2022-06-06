#![allow(dead_code)]

use super::{BTProtocol, Role, Server, Torrent};

pub struct Seeder {
    server_side: Server<BTProtocol>,
    torrent: Torrent,
}

impl Role for Seeder {
    fn new(torrent_pathname: &str) -> Self {
        Self {
            torrent: Torrent::new_from_pathname(torrent_pathname),
            server_side: Server::new(BTProtocol),
        }
    }

    fn deactivate(&mut self) {}

    fn activate(&mut self) {}
}
