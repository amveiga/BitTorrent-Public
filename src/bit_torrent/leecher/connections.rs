use std::collections::HashMap;
use std::thread;

#[derive(Default)]

pub struct Connections {
    connections: HashMap<String, thread::JoinHandle<()>>,
}

impl Connections {
    pub fn new() -> Self {
        Self {
            connections: HashMap::default(),
        }
    }

    pub fn add(&mut self, ip: String, handle: thread::JoinHandle<()>) {
        log::info!("Connections - open connections: {}", self.connections.len());
        self.connections.insert(ip, handle);
    }

    pub fn remove(&mut self, ip: &str) {
        self.connections.remove(ip);
    }
}
