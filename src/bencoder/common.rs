use std::collections::HashMap;
use std::collections::LinkedList;

pub const START_INTEGER: u8 = b'i';
pub const START_STRING: u8 = b':';
pub const START_LIST: u8 = b'l';
pub const START_DICT: u8 = b'd';
pub const END: u8 = b'e';

#[derive(Debug)]
pub enum Types {
    Integer(i64),
    String(Vec<u8>),
    Dictionary(HashMap<Vec<u8>, Types>),
    List(LinkedList<Types>),
}

impl Types {
    pub fn get_integrer(&self) -> i64 {
        match &self {
            Types::Integer(int) => *int,
            _ => unreachable!(),
        }
    }

    pub fn get_string(&self) -> Vec<u8> {
        match &self {
            Types::String(str) => str.clone(),
            _ => unreachable!(),
        }
    }

    pub fn get_dictionary(&self) -> &HashMap<Vec<u8>, Types> {
        match &self {
            Types::Dictionary(dict) => dict,
            _ => unreachable!(),
        }
    }

    pub fn get_list(&self) -> &LinkedList<Types> {
        match &self {
            Types::List(list) => list,
            _ => unreachable!(),
        }
    }
}
