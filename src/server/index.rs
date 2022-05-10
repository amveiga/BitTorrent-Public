use std::fmt;
use std::io::{self, ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::sync::mpsc;
use std::thread;

use super::constants;

pub struct Server {
    pub address: SocketAddr,
}

impl Server {
    pub fn start(ip: String, port: String) -> Server {
        let address: SocketAddr = format!("{}:{}", ip, port)
            .parse()
            .expect("Unable to parse socket address");
        Server { address }
    }
    pub fn run(&mut self) -> io::Result<()> {
        let listener = TcpListener::bind(self.address).expect("Listener failed to bind");
        listener
            .set_nonblocking(true)
            .expect("Failed to initialize non-blocking");

        let mut clients = Vec::new();
        let (tx, _rx) = mpsc::channel::<String>();
        loop {
            if let Ok((mut socket, addr)) = listener.accept() {
                println!("Client {} is connected", addr);

                let tx = tx.clone();
                clients.push(socket.try_clone().expect("Failed to clone client"));

                thread::spawn(move || loop {
                    let mut buff = vec![0; constants::MSG_SIZE];

                    match socket.read_exact(&mut buff) {
                        Ok(_) => {
                            let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                            let msg = String::from_utf8(msg).expect("Invalid utf8 message");

                            println!("{} says: {:?}", addr, msg);
                            let mut buff = msg.clone().into_bytes();
                            tx.send(msg).expect("Failed to send msg to rx");
                            buff.resize(constants::MSG_SIZE, 0);
                            socket.write_all(&buff).unwrap();
                        }
                        Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                        Err(_) => {
                            println!("Closing connection with: {}", addr);
                            break;
                        }
                    }

                    sleep();
                });
            }
            sleep();
        }
    }
    pub fn stop(&self) -> io::Result<()> {
        Ok(())
    }
    pub fn ip(&self) -> String {
        self.address.ip().to_string()
    }
    pub fn port(&self) -> String {
        self.address.port().to_string()
    }
}

impl fmt::Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.ip(), self.port())
    }
}

impl PartialEq for Server {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address
    }
}

fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_creates_a_server() {
        let ip = String::from("127.0.0.1");
        let port = String::from("7878");
        let address: SocketAddr = format!("{}:{}", ip, port)
            .parse()
            .expect("Unable to parse socket address");
        let server = Server::start(ip, port);

        assert_eq!(server.address, address)
    }
}
