#![allow(dead_code)]

use super::common::*;

use std::collections::HashMap;
use std::collections::LinkedList;
use std::error;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::Read;
use std::str;

pub struct Decoder {
    to_decode: Vec<u8>,
    len: usize,
    pos: usize,
}

impl Decoder {
    pub fn new_from_string(data: String) -> Self {
        let to_decode = data.as_bytes().to_owned();
        let len = data.len();
        Self {
            to_decode,
            len,
            pos: 0,
        }
    }

    pub fn new_from_bytes(to_decode: &[u8]) -> Self {
        let len = to_decode.len();
        Self {
            to_decode: to_decode.to_owned(),
            len,
            pos: 0,
        }
    }

    pub fn new_from_file(mut file: File) -> Result<Self, io::Error> {
        let len = file.metadata()?.len() as usize;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(Self {
            len,
            to_decode: buffer,
            pos: 0,
        })
    }

    pub fn decode(&mut self) -> Result<Types, Box<dyn error::Error>> {
        let mut message = self.decode_next()?;

        while !self.end() {
            message = self.decode_next()?;
        }
        Ok(message)
    }

    fn end(&self) -> bool {
        self.len <= self.pos
    }

    fn decode_next(&mut self) -> Result<Types, Box<dyn error::Error>> {
        match self.to_decode[self.pos] {
            START_INTEGER => self.decode_integer(),
            START_LIST => self.decode_list(),
            START_DICT => self.decode_dictionary(),
            b'0'..=b'9' => self.decode_string(),
            _ => Err(Box::new(fmt::Error)),
        }
    }

    fn decode_integer(&mut self) -> Result<Types, Box<dyn error::Error>> {
        self.pos += 1;
        let slice_start = self.pos;

        while self.to_decode[self.pos] != END {
            self.pos += 1;
        }

        let slice_end = self.pos;
        self.pos += 1;

        let parsed_number = str::from_utf8(&self.to_decode[slice_start..slice_end])?.parse()?;
        Ok(Types::Integer(parsed_number))
    }

    fn decode_string(&mut self) -> Result<Types, Box<dyn error::Error>> {
        let mut len = 0;

        while self.to_decode[self.pos + len] != START_STRING {
            len += 1;
        }

        let len_string: usize =
            str::from_utf8(&self.to_decode[self.pos..self.pos + len])?.parse()?;

        self.pos += len + 1; // Because we skip the len and the ":"

        let s = self.to_decode[self.pos..self.pos + len_string].to_vec();

        self.pos += len_string;

        Ok(Types::String(s))
    }

    fn decode_list(&mut self) -> Result<Types, Box<dyn error::Error>> {
        let mut list = LinkedList::new();
        self.pos += 1;

        while self.to_decode[self.pos] != END {
            let parsed_item = self.decode_next()?;
            list.push_back(parsed_item);
        }
        self.pos += 1;

        Ok(Types::List(list))
    }

    fn decode_dictionary(&mut self) -> Result<Types, Box<dyn error::Error>> {
        let mut dict = HashMap::new();
        self.pos += 1;

        while self.to_decode[self.pos] != END {
            let key = match self.decode_next()? {
                Types::String(key) => key,
                _ => return Err(Box::new(fmt::Error)),
            };

            let value = self.decode_next()?;

            dict.insert(key, value);
        }
        self.pos += 1;

        Ok(Types::Dictionary(dict))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1_parse_integer_correctly() {
        let bencoded_message = "i2510e".to_string();
        let mut bencoder = Decoder::new_from_string(bencoded_message);
        let parsed_message = bencoder
            .decode()
            .expect("Error in test-1: Unable to parse the integer.");
        assert_eq!(
            parsed_message
                .get_integrer()
                .expect("Error in test-1: Unable to get integrer from decoded message"),
            2510
        );
    }

    #[test]
    fn test2_parse_string_correctly() {
        let bencoded_message = "4:Hola".to_string();
        let mut bencoder = Decoder::new_from_string(bencoded_message);
        let parsed_message = bencoder
            .decode()
            .expect("Error in test-2: Unable to parse the string.");
        assert_eq!(
            parsed_message
                .get_string()
                .expect("Error in test-2: Unable to get string from decoded message"),
            b"Hola"
        );
    }

    #[test]
    fn test3_decode_list_correctly() {
        let bencoded_message = "l4:spam4:eggse".to_string();
        let mut bencoder = Decoder::new_from_string(bencoded_message);
        let decoded_message = bencoder
            .decode()
            .expect("Error in test-3: Unable to parse the list.");

        let decode_list = decoded_message
            .get_list()
            .expect("Error in test-3: Unable to get list from decoded message");
        let mut expected_list = LinkedList::new();
        expected_list.push_back(b"spam".to_vec());
        expected_list.push_back(b"eggs".to_vec());
        let mut expected_list_iter = expected_list.iter();
        let decode_list_iter = decode_list.iter();
        assert_eq!(decode_list.len(), expected_list.len());
        for m in decode_list_iter {
            assert_eq!(
                &m.get_string()
                    .expect("Error test-3: Unable to get string from list "),
                expected_list_iter
                    .next()
                    .expect("Error test-3: Unable to iterate")
            );
        }
    }

    #[test]
    fn test4_parse_dictionaries_correctly() {
        let bencoded_message =
            "d9:publisher3:bob17:publisher-webpage15:www.example.com18:publisher.location4:homee"
                .to_string();
        let mut bencoder = Decoder::new_from_string(bencoded_message);
        let mut expected_dict = HashMap::new();
        expected_dict.insert(b"publisher".to_vec(), b"bob".to_vec());
        expected_dict.insert(b"publisher.location".to_vec(), b"home".to_vec());
        expected_dict.insert(b"publisher-webpage".to_vec(), b"www.example.com".to_vec());
        let returned_dict = match bencoder
            .decode()
            .expect("Error in test-4: Unable to parse the dictionary.")
        {
            Types::Dictionary(dict) => dict,
            _ => HashMap::new(),
        };
        assert_eq!(expected_dict.len(), returned_dict.len());
        for (key, val) in returned_dict.iter() {
            assert_eq!(
                &val.get_string()
                    .expect("test-5: Unable to get string from dict"),
                expected_dict
                    .get(key)
                    .expect("test-5: Unable to get value from expected dictionary")
            )
        }
    }

    #[test]
    fn test5_doesnt_parse_wrong_format() {
        let bencoded_message =
            "d:publisher3:bob17:publisher-webpage15:www.example.com18:publisher.location4:homee"
                .to_string();
        let mut bencoder = Decoder::new_from_string(bencoded_message);
        assert!(bencoder.decode().is_err());
    }
}
