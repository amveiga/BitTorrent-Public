#![allow(dead_code)]

use super::Role;

pub struct Process<T: Role> {
    role: T,
}

impl<T: Role> Process<T> {
    pub fn new(torrent_pathname: &str) -> Self {
        let mut role = T::new(torrent_pathname);

        role.activate();

        Self { role }
    }
}
