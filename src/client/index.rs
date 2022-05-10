use std::fmt;
use std::io::{self, ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

use super::constants;

pub struct Client {
    pub address: SocketAddr,
}

impl Client {
    pub fn start(ip: String, port: String) -> Client {
        let address: SocketAddr = format!("{}:{}", ip, port)
            .parse()
            .expect("Unable to parse socket address");
        Client { address }
    }
    pub fn run(&self) -> io::Result<()> {
        let mut connection = TcpStream::connect(self.address).expect("Stream failed to connect");
        connection
            .set_nonblocking(true)
            .expect("Failed to initiate non-blocking");

        let (tx, rx) = mpsc::channel::<String>();

        thread::spawn(move || loop {
            let mut buff = vec![0; constants::MSG_SIZE];
            match connection.read_exact(&mut buff) {
                Ok(_) => {
                    let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                    let msg = String::from_utf8(msg).expect("Invalid utf8 message");
                    println!("Response: {:?}", msg);
                }
                Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                Err(_) => {
                    println!("Connection with server was lost");
                    break;
                }
            }

            match rx.try_recv() {
                Ok(msg) => {
                    let mut buff = msg.clone().into_bytes();
                    buff.resize(constants::MSG_SIZE, 0);
                    connection
                        .write_all(&buff)
                        .expect("writing to socket failed");
                    println!("Message sent: {:?}", msg);
                }
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => break,
            }

            thread::sleep(Duration::from_millis(100));
        });

        println!("Write a Message:");
        loop {
            let mut buff = String::new();
            io::stdin()
                .read_line(&mut buff)
                .expect("Reading from stdin failed");
            let msg = buff.trim().to_string();
            if msg == ":quit" || tx.send(msg).is_err() {
                break;
            }
        }
        println!("Disconected");
        Ok(())
    }
    pub fn ip(&self) -> String {
        self.address.ip().to_string()
    }
    pub fn port(&self) -> String {
        self.address.port().to_string()
    }
}

impl fmt::Display for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.ip(), self.port())
    }
}
