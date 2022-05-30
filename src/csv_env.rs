use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

/// Set environment variables from a configuration file.
///
/// File needs to be .CSV with KEY,VALUE rows only.
///
/// Config location:
///
/// src/configs/{name}.csv
///
/// Params:
///
/// name: String -> environment file name

pub fn set_env(name: String) {
    let filename = format!("src/configs/{name}.csv");
    let file = File::open(filename).expect("Failed to open file");
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.expect("Failed to read line");

        let key_value = line.split(',');
        let vec: Vec<&str> = key_value.collect();

        if vec.len() > 2 {
            println!("Invalid config! config must be KEY,VALUE {:?}", vec);
        } else {
            env::set_var(vec[0].to_uppercase(), vec[1]);
        }
    }
}
