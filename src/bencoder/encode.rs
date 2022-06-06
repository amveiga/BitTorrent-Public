#![allow(dead_code)]

use super::common::*;
use std::{collections::HashMap, collections::LinkedList, error, str::from_utf8};

pub struct Encoder {
    to_encode: Types,
}

impl Encoder {
    pub fn new(to_encode: Types) -> Self {
        Self { to_encode }
    }

    pub fn encode(&self) -> Result<Vec<u8>, Box<dyn error::Error>> {
        let encoded_message = self.match_encode(&self.to_encode)?;
        Ok(encoded_message)
    }

    fn match_encode(&self, t: &Types) -> Result<Vec<u8>, Box<dyn error::Error>> {
        match t {
            Types::Integer(int) => self.encode_integer(*int),
            Types::String(s) => self.encode_string(s),
            Types::List(l) => self.encode_list(l),
            Types::Dictionary(d) => self.encode_dictionary(d),
        }
    }

    fn encode_integer(&self, int: i64) -> Result<Vec<u8>, Box<dyn error::Error>> {
        Ok(format!("i{}e", int).as_bytes().to_vec())
    }

    fn encode_string(&self, str: &[u8]) -> Result<Vec<u8>, Box<dyn error::Error>> {
        Ok(format!("{}:{}", str.len(), from_utf8(str)?)
            .as_bytes()
            .to_vec())
    }

    fn encode_list(&self, list: &LinkedList<Types>) -> Result<Vec<u8>, Box<dyn error::Error>> {
        let mut encoded_list: Vec<u8> = vec![b'l'];
        for item in list {
            self.match_encode(item)?;
        }
        encoded_list.push(b'e');
        Ok(encoded_list)
    }

    fn encode_dictionary(
        &self,
        dict: &HashMap<Vec<u8>, Types>,
    ) -> Result<Vec<u8>, Box<dyn error::Error>> {
        let mut encoded_dict: Vec<u8> = vec![b'd'];
        for (key, val) in dict {
            encoded_dict.extend_from_slice(&self.encode_string(key)?);
            self.match_encode(val)?;
        }
        encoded_dict.push(b'e');
        Ok(encoded_dict)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1_encode_integrer_correctly() {
        let bencoder = Encoder::new(Types::Integer(10));
        let encoded_message = bencoder
            .encode()
            .expect("Error in test-1: Unable to encode the integer.");
        assert_eq!(encoded_message, b"i10e");
    }

    #[test]
    fn test2_encode_string_correctly() {
        let bencoder = Encoder::new(Types::String(b"Hola".to_vec()));
        let encoded_message = bencoder
            .encode()
            .expect("Error in test-2: Unable to encode the string.");

        assert_eq!(encoded_message, b"4:Hola");
    }

    #[test]
    fn test3_encode_list_correctly() {
        let list = LinkedList::from([
            Types::String(b"Hola".to_vec()),
            Types::String(b"Chau".to_vec()),
        ]);
        let bencoder = Encoder::new(Types::List(list));
        bencoder
            .encode()
            .expect("Error in test-3: Unable to encode the list.");
    }
    #[test]
    fn test4_encode_dictionary_correctly() {
        let mut dict = HashMap::new();
        dict.insert(b"publisher".to_vec(), Types::String(b"bob".to_vec()));
        dict.insert(
            b"publisher.location".to_vec(),
            Types::String(b"home".to_vec()),
        );
        dict.insert(
            b"publisher-webpage".to_vec(),
            Types::String(b"www.example.com".to_vec()),
        );

        let bencoder = Encoder::new(Types::Dictionary(dict));
        bencoder
            .encode()
            .expect("Error in test-4: Unable to encode the dictionary.");
    }
}
