use crate::file_system::get_sorted_paths;

use super::Fragment;
use std::fs::{self, remove_dir_all, remove_file, File as Handler};

use std::convert::AsRef;
use std::io::{Error, Read, Write};
use std::path::{Path, PathBuf};

pub struct File {
    pathname: String,
    handler: Handler,
}

impl File {
    pub fn open_file<T: AsRef<Path>>(pathname: &T) -> Result<Handler, Error> {
        Handler::options()
            .read(true)
            .write(true)
            .create(true)
            .open(pathname)
    }

    pub fn new<T: AsRef<Path>>(pathname: T) -> Self {
        let handler = File::open_file(&pathname).expect("Error opening file");

        Self {
            handler,
            pathname: pathname
                .as_ref()
                .to_str()
                .expect("Error converting to str")
                .to_owned(),
        }
    }

    pub fn with_contents<T: AsRef<Path>, B: AsRef<[u8]>>(pathname: T, contents: B) -> Self {
        let mut handler = File::open_file(&pathname).expect("Error opening file");
        handler.write_all(contents.as_ref()).expect("Writing error");

        Self {
            handler,
            pathname: pathname
                .as_ref()
                .to_str()
                .expect("Error converting to str")
                .to_owned(),
        }
    }

    pub fn get_contents(&mut self) -> Vec<u8> {
        let mut handler = File::open_file(&self.pathname).expect("Error opening file");

        let mut contents = vec![];

        handler
            .read_to_end(&mut contents)
            .expect("Failed reading from handler");

        contents
    }

    pub fn get_chunk(&mut self, chunk_size: usize, chunk_number: usize) -> Fragment {
        let handler = File::open_file(&self.pathname).expect("Error opening file");

        let file_as_bytes = (handler).bytes();

        let mut bytes: Vec<u8> = Vec::new();

        let first_byte = chunk_number * chunk_size;
        let last_byte = first_byte + chunk_size;

        for (byte_number, byte) in file_as_bytes.enumerate() {
            if let Ok(byte_value) = byte {
                if byte_number >= first_byte && byte_number < last_byte {
                    bytes.push(byte_value)
                }
            }
        }

        Fragment::from(bytes)
    }

    pub fn new_file_from_piece<T: AsRef<Path>>(piece: &[u8], pathname: T) -> Result<Self, Error> {
        let pathname = pathname
            .as_ref()
            .to_str()
            .expect("Error converting to str")
            .to_owned();

        let dir = fs::create_dir_all(format!(
            "./download_temp/{}",
            pathname.split(".piece").collect::<Vec<&str>>()[0]
        ));
        if dir.is_err() {}

        let mut handler = File::open_file(&format!(
            "./download_temp/{}/{}",
            pathname.split(".piece").collect::<Vec<&str>>()[0],
            pathname
        ))?;
        handler.write_all(piece)?;

        Ok(Self { handler, pathname })
    }

    pub fn concat(&mut self, piece: PathBuf) -> Result<(), Error> {
        let piece_file = File::new(&piece).get_contents();
        self.handler.write_all(&piece_file)?;
        remove_file(piece)?;
        Ok(())
    }

    pub fn join_pieces<T: AsRef<Path>>(path_from: T, path_to: T) -> Result<(), Error> {
        let sorted_path = get_sorted_paths(path_from.as_ref().to_str().unwrap());
        let mut new_file = Self::new(&path_to);

        for piece_path in sorted_path {
            new_file.concat(piece_path)?
        }
        remove_dir_all(path_from.as_ref())?;
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::fs::{create_dir_all, remove_dir, remove_file};

//     #[test]
//     fn test_create_dir() {
//         let dir = fs::create_dir_all("./pieces");
//         if dir.is_err() {}
//         File::open_file(&format!("./pieces/{}", "pathname.1")).unwrap();
//         let dir = fs::create_dir_all("./pieces");
//         if dir.is_err() {}
//         File::open_file(&format!("./pieces/{}", "pathname.2")).unwrap();
//     }
//     #[test]
//     fn creates_file_correctly() {
//         File::new("empty1.txt");

//         let mut test_file = Handler::open("empty1.txt").expect("Error opening file");
//         let mut contents = String::new();

//         test_file
//             .read_to_string(&mut contents)
//             .expect("Failed to read file - Test: creates_file_correctly");

//         remove_file("empty1.txt").expect("Failed to delete file - Test: creates_file_correctly");

//         assert_eq!(contents, "");
//     }

//     #[test]
//     fn obtains_chunks_correctly() {
//         let mut file = File::with_contents("hey.txt", b"Hey!");

//         let chunk1 = file.get_chunk(2, 0);

//         let chunk2 = file.get_chunk(2, 1);

//         assert_eq!(chunk1.get_content(), b"He");
//         assert_eq!(chunk2.get_content(), b"y!");

//         remove_file("hey.txt").expect("Failed to delete file - Test: obtains_chunks_correctly");
//     }

//     #[test]
//     fn recreates_file_from_chunks_correctly() {
//         let mut own_test_file = File::with_contents("test.txt", b"Hey!");
//         let mut result_file = File::new("test_cpy.txt");

//         for chunk_number in 0..own_test_file.get_contents().chars().count() {
//             let chunk = own_test_file.get_chunk(1, chunk_number);

//             result_file
//                 .concat(chunk)
//                 .expect("Error in concating chunk");
//         }

//         assert_eq!(own_test_file.get_contents(), result_file.get_contents());
//         remove_file("test_cpy.txt")
//             .expect("Failed to delete file - Test: recreates_file_from_chunks_correctly");
//         remove_file("test.txt")
//             .expect("Failed to delete file - Test: recreates_file_from_chunks_correctly");
//     }
