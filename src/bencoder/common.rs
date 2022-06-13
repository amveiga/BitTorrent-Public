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
    pub fn get_integrer(&self) -> Option<i64> {
        match &self {
            Types::Integer(int) => Some(*int),
            _ => None,
        }
    }

    pub fn get_string(&self) -> Option<Vec<u8>> {
        match &self {
            Types::String(str) => Some(str.clone()),
            _ => None,
        }
    }

    pub fn get_dictionary(&self) -> Option<&HashMap<Vec<u8>, Types>> {
        match &self {
            Types::Dictionary(dict) => Some(dict),
            _ => None,
        }
    }

    pub fn get_list(&self) -> Option<&LinkedList<Types>> {
        match &self {
            Types::List(list) => Some(list),
            _ => None,
        }
    }
}
