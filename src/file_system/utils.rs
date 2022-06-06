use std::fs::read_dir;
use std::path::PathBuf;

pub fn get_sorted_paths(dir: &str) -> Vec<PathBuf> {
    let mut paths = read_dir(dir)
        .expect("Error reading from path.")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect::<Vec<_>>();

    paths.sort();

    paths
}
