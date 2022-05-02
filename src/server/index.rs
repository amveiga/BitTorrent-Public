use std::cell::Cell;
pub struct Server {
    id: usize,
    pub name: String,
}

thread_local!(static SERVER_ID: Cell<usize> = Cell::new(0));

impl Server {
    pub fn new(name: String) -> Server {
        SERVER_ID.with(|thread_id| {
            let id = thread_id.get();
            thread_id.set(id + 1);
            Server { id, name }
        })
    }
    pub fn id(&self) -> String {
        self.id.to_string()
    }
    pub fn name(&self) -> String {
        self.name.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_creates_a_server() {
        let name = String::from("PEPE");
        let server = Server::new(name);
        assert_eq!("PEPE", server.name())
    }
    #[test]
    fn multiple_servers_have_different_ids() {
        let name = String::from("PEPE");
        let server = Server::new(name);

        let name = String::from("JOSE");
        let server2 = Server::new(name);

        assert_ne!(server.id(), server2.id())
    }
}
