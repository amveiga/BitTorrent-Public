/// A logger that prints messages to the console or a specified file.
/// The logger is a singleton, so only one instance can be created, will panic if instanced more than once.
/// The logger is activated by calling the `activate` function with optional parameters such as path of file and
/// the maximun level desired to log. Returns a result, only error may be SetLoggerError
///
/// # Examples
/// Initialize loger with default parameters:
///     - if no path is provided (`None`), the logger will print to the console
///     - if no level is provided (`None`), the logger will log all messages
/// ```rust
///     use sitos::logger::Logger;
///     Logger::activate(None, None).unwrap();
///
///     log::info!("Hello, world!");
/// ```
/// If you want to log to a file, you can provide the path of the file:
/// ```rust
///     use sitos::logger::Logger;
///     Logger::activate(Some("log.txt".to_string()), None).unwrap();
/// ```
///
/// If you want to log only messages of a certain level or lower, you can provide the level:
/// ```rust
///     use sitos::logger::Logger;
///     Logger::activate(None, Some(log::LevelFilter::Info)).unwrap();
/// ```
///
/// # Notes
/// The order of levels are:
///     - `Error`
///     - `Warn`
///     - `Info`
///     - `Debug`
///     - `Trace`
/// Being error the lowest and trace the maximun level of logging.
///
extern crate log;

use std::{
    io::Write,
    sync::{
        mpsc::{self, Receiver, Sender},
        Mutex,
    },
    thread,
    time::Instant,
};

use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};

pub struct Logger {
    tx: Mutex<Sender<String>>,
    start: Instant,
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() >= Level::Error
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let tx = self
                .tx
                .lock()
                .expect("log: lock failed, another thread panicked");
            let now = self.start.elapsed();
            let seconds = now.as_secs();
            let hours = seconds / 3600;
            let minutes = (seconds / 60) % 60;
            let seconds = seconds % 60;
            let miliseconds = now.subsec_millis();

            let _ = tx.send(format!(
                "[{:02}:{:02}:{:02}.{:03}] {:6} [{}:{}] {}\n",
                hours,
                minutes,
                seconds,
                miliseconds,
                record.level(),
                record.module_path().unwrap_or("-"),
                record.line().unwrap_or(0),
                record.args()
            ));
        }
    }

    fn flush(&self) {}
}

impl Logger {
    fn init(
        boxed_logger: Box<Logger>,
        max_level_filter: Option<LevelFilter>,
    ) -> Result<(), SetLoggerError> {
        match max_level_filter {
            Some(level_filter) => log::set_max_level(level_filter),
            None => log::set_max_level(LevelFilter::Trace),
        }
        log::set_boxed_logger(boxed_logger)
    }

    fn new(tx: Sender<String>) -> Self {
        Self {
            tx: Mutex::new(tx),
            start: Instant::now(),
        }
    }

    pub fn activate(
        path: Option<String>,
        max_level_filter: Option<LevelFilter>,
    ) -> Result<(), SetLoggerError> {
        let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();

        thread::spawn(move || {
            match path {
                Some(path) => {
                    let mut file = std::fs::OpenOptions::new()
                        .write(true)
                        .create(true)
                        .open(path)
                        .expect("log: failed to open log file");
                    for received in rx {
                        file.write_all(received.as_bytes())
                            .expect("log: failed to write to log file");
                    }
                }
                None => {
                    for received in rx {
                        print!("{}", received);
                    }
                }
            };
        });

        let new_logger = Self::new(tx);

        Self::init(Box::new(new_logger), max_level_filter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread::sleep, time::Duration};

    #[test]
    fn test_simple_logger_with_file() {
        Logger::activate(Some("test.log".to_string()), None).unwrap();

        sleep(Duration::from_secs(1));
        log::error!("El sito les informa");
        sleep(Duration::from_secs(1));
        log::warn!("Que hay un warn");
        sleep(Duration::from_secs(1));
        log::info!("En mi casa hay una info");
        sleep(Duration::from_secs(1));
        log::debug!("Y en mi corazon un debug");
        sleep(Duration::from_secs(1));
        log::trace!("Que pueda trazar mi camino");
    }
}
