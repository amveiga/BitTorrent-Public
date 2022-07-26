use std::io;
use std::sync::mpsc::{self, Receiver, Sender};

use log::LevelFilter;
use sitos::bit_torrent::Tracker;
use sitos::csv_env as bitEnv;
use sitos::logger::Logger;

fn main() {
    bitEnv::set_env(String::from("congif"));
    Logger::activate(
        Some("torrentsito.log".to_string()),
        Some(LevelFilter::Debug),
    )
    .unwrap();

    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    if let Ok(tracker) = Tracker::new() {
        let tracker_handler = tracker.activate(rx);

        loop {
            let mut buffer = String::new();
            log::debug!("Waiting to recieve from stdin");
            io::stdin().read_line(&mut buffer).unwrap();
            log::debug!("Recieved from stdin");

            if buffer.trim() == "exit" {
                tx.send(buffer).unwrap();
                log::debug!("exit message sent");
                break;
            }
        }
        log::debug!("Trying to join tracker thread");
        tracker_handler.join().unwrap();
        log::debug!("Successfully joined tracker thread")
    } else {
        panic!("Failed to create tracker");
    }
}
