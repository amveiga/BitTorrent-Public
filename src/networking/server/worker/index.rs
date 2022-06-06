#![allow(dead_code)]

use super::Job;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Worker {
    thread_handle: thread::JoinHandle<()>,
}

impl Worker {
    pub fn new(receiver: Arc<Mutex<Receiver<Job>>>) -> Self {
        let thread_handle = thread::spawn(move || loop {
            let job = receiver
                .lock()
                .expect("Failed to lock")
                .recv()
                .expect("Failed to recive message");

            job();
        });

        Self { thread_handle }
    }
}
