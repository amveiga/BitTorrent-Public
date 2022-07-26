use std::{
    sync::{mpsc::Receiver, Arc, Mutex},
    thread::{self, JoinHandle},
};

#[derive(PartialEq)]
pub enum TrackerStatus {
    Active,
    Break,
}

impl TrackerStatus {
    pub fn await_signal(
        state: Arc<Mutex<TrackerStatus>>,
        remove_rx: Receiver<String>,
    ) -> JoinHandle<()> {
        thread::spawn(move || loop {
            if TrackerStatus::Break == *state.lock().unwrap() {
                break;
            }

            let maybe_recv = remove_rx.try_recv();
            if let Ok(recv) = maybe_recv {
                log::debug!("TrackerStatus::await_signal() - recieved message {}", recv);
                if recv.trim() == "exit" {
                    log::debug!(
                        "TrackerStatus::await_signal() - setting state to TrackerStatus::Break"
                    );
                    *state.lock().unwrap() = TrackerStatus::Break;
                    log::debug!(
                        "TrackerStatus::await_signal() - state to TrackerStatus::Break set"
                    );
                    break;
                }
            }

            thread::yield_now();
        })
    }
}
