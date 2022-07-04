use log::LevelFilter;
use sitos::bit_torrent::BitTorrent;
use sitos::csv_env::set_env;
use sitos::logger::Logger;
use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        2 => {
            set_env(String::from("default"));
        }
        3 => {
            let env_type = String::from(&args[2]);
            set_env(env_type);
        }
        _ => {
            eprintln!("Error! Wrong arguments use: <.TORRENT PATH>, optional: <ENV>");
            process::exit(1);
        }
    }

    Logger::activate(Some("sitos.log".to_string()), Some(LevelFilter::Trace)).unwrap();

    let mut bit_torrent_instance = BitTorrent::new();

    bit_torrent_instance.new_leecher(&args[1]);
}
