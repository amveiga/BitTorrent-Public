pub use super::{
    networking::utils::get_available_port, Connection, Ledger, TrackerError, TrackerStatus,
};
use std::io;
use std::net::TcpListener;
use std::sync::mpsc::Receiver;
pub use std::sync::{Arc, Mutex};
pub use std::thread;

pub struct Tracker {
    ledger: Arc<Mutex<Ledger>>,
    listener: TcpListener,
    status: Arc<Mutex<TrackerStatus>>,
}

impl Tracker {
    pub fn new() -> Result<Self, TrackerError> {
        let listener = TcpListener::bind(String::from("127.0.0.1:8080"))
            .or(Err(TrackerError::FailedToBindStream))?;

        listener
            .set_nonblocking(true)
            .expect("Cannot set non-blocking");

        Ok(Self {
            listener,
            ledger: Arc::new(Mutex::new(Ledger::new())),
            status: Arc::new(Mutex::new(TrackerStatus::Active)),
        })
    }

    pub fn activate(self, rx: Receiver<String>) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let mut connections: Vec<thread::JoinHandle<()>> =
                vec![TrackerStatus::await_signal(Arc::clone(&self.status), rx)];

            for stream in self.listener.incoming() {
                match stream {
                    Ok(s) => {
                        if let Ok(mut ledger_guard) = self.ledger.lock() {
                            ledger_guard.check();
                        } else {
                            log::error!("Tracker::activate() - Failed to lock ledger");
                            break;
                        }
                        connections.push(
                            Connection::new(s, Arc::clone(&self.ledger), Arc::clone(&self.status))
                                .activate(),
                        );
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        if TrackerStatus::Break == *self.status.lock().unwrap() {
                            println!("break - ErrorKind::WouldBlock");

                            log::debug!("Tracker::activate(): breaking listener loop");
                            break;
                        } else {
                            thread::yield_now();
                            continue;
                        }
                    }
                    Err(e) => panic!("encountered IO error: {e}"),
                }
            }

            for connection in connections {
                connection
                    .join()
                    .expect("Failed to join thread: Tracker connection");
            }
        })
    }
}
