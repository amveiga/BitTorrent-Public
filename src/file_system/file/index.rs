use crate::file_system::get_sorted_paths;

use std::env;
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

    pub fn get_block(
        &mut self,
        piece_index: usize,
        piece_size: usize,
        block_size: usize,
        block_offset: usize,
    ) -> Vec<u8> {
        let handler = File::open_file(&self.pathname).expect("Error opening file");

        let file_as_bytes = handler.bytes();

        let mut bytes: Vec<u8> = Vec::new();

        let first_byte = piece_index * piece_size + block_offset;
        let last_byte = first_byte + block_size;

        for (byte_number, byte) in file_as_bytes.enumerate() {
            if let Ok(byte_value) = byte {
                if byte_number >= first_byte && byte_number < last_byte {
                    bytes.push(byte_value)
                }
            }
        }

        bytes
    }

    pub fn new_file_from_piece<T: AsRef<Path>>(piece: &[u8], pathname: T) -> Result<Self, Error> {
        let pathname = pathname
            .as_ref()
            .to_str()
            .expect("Error converting to str")
            .to_owned();

        let dir = fs::create_dir_all(format!(
            "{}/{}",
            env::var("TEMP_PATH")
                .unwrap_or_else(|_| "".to_string())
                .as_str(),
            pathname.split(".piece").collect::<Vec<&str>>()[0]
        ));
        if dir.is_err() {}

        let mut handler = File::open_file(&format!(
            "{}/{}/{}",
            env::var("TEMP_PATH")
                .unwrap_or_else(|_| "".to_string())
                .as_str(),
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

        let path_to = path_to
            .as_ref()
            .to_str()
            .expect("Error converting to str")
            .to_owned();

        let mut split: Vec<&str> = path_to.split('/').collect();

        split.pop();

        let dir = split.join("/");

        fs::create_dir_all(dir)?;

        let mut new_file = Self::new(&path_to);

        for piece_path in sorted_path {
            new_file.concat(piece_path)?
        }

        remove_dir_all(path_from.as_ref())?;
        Ok(())
    }
}
