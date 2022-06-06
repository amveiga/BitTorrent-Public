pub use super::{HTTPTracker, Protocol, Server};

pub struct Tracker {
    handler: Server<HTTPTracker>,
}

impl Tracker {
    pub fn new(ip: &str, port: u16) -> Self {
        Self {
            handler: Server::new_from_socket(HTTPTracker::new(None), ip, port),
        }
    }

    pub fn run(&mut self) {
        self.handler.run();
    }
}
