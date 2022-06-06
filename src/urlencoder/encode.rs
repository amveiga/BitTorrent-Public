#![allow(dead_code)]

use super::reserved_characters::*;

pub struct UrlEncoder {
    url: Vec<u8>,
}

impl UrlEncoder {
    pub fn new(url_to_encode: Vec<u8>) -> Self {
        Self { url: url_to_encode }
    }

    pub fn encode_binary_data(to_encode: Vec<u8>) -> String {
        let mut new_url = Vec::new();
        for c in &to_encode {
            new_url.extend_from_slice(&match *c {
                b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'-' | b'~' | b'.' | b'_' => vec![*c],
                _ => format!("%{:02X}", *c).bytes().collect(),
            });
        }
        String::from_utf8(new_url).expect("Failed to encode binary data")
    }

    pub fn encode(to_encode: Vec<u8>) -> Vec<u8> {
        let mut new_url = Vec::new();
        for c in &to_encode {
            new_url.extend_from_slice(&match *c {
                CHARACTER_SPACE => SPECIAL_CHARACTER_SPACE.to_vec(),
                CHARACTER_PERCENTAGE => SPECIAL_CHARACTER_PERCENTAGE.to_vec(),
                CHARACTER_LOWER => SPECIAL_CHARACTER_LOWER.to_vec(),
                CHARACTER_HIGHER => SPECIAL_CHARACTER_HIGHER.to_vec(),
                CHARACTER_HASHTAG => SPECIAL_CHARACTER_HASHTAG.to_vec(),
                CHARACTER_PIPE => SPECIAL_CHARACTER_PIPE.to_vec(),
                CHARACTER_DQM => SPECIAL_CHARACTER_DQM.to_vec(),
                CHARACTER_EM => SPECIAL_CHARACTER_EM.to_vec(),
                CHARACTER_ASTERISTIC => SPECIAL_CHARACTER_ASTERISTIC.to_vec(),
                CHARACTER_SQM => SPECIAL_CHARACTER_SQM.to_vec(),
                CHARACTER_OP => SPECIAL_CHARACTER_OP.to_vec(),
                CHARACTER_CP => SPECIAL_CHARACTER_CP.to_vec(),
                CHARACTER_COLON => SPECIAL_CHARACTER_COLON.to_vec(),
                CHARACTER_SEMICOLON => SPECIAL_CHARACTER_SEMICOLON.to_vec(),
                CHARACTER_ATSIGN => SPECIAL_CHARACTER_ATSIGN.to_vec(),
                CHARACTER_AMPERSAND => SPECIAL_CHARACTER_AMPERSAND.to_vec(),
                CHARACTER_EQUAL => SPECIAL_CHARACTER_EQUAL.to_vec(),
                CHARACTER_PLUS => SPECIAL_CHARACTER_PLUS.to_vec(),
                CHARACTER_PS => SPECIAL_CHARACTER_PS.to_vec(),
                CHARACTER_COMMA => SPECIAL_CHARACTER_COMMA.to_vec(),
                CHARACTER_SLASH => SPECIAL_CHARACTER_SLASH.to_vec(),
                CHARACTER_QM => SPECIAL_CHARACTER_QM.to_vec(),
                CHARACTER_OSB => SPECIAL_CHARACTER_OSB.to_vec(),
                CHARACTER_CSB => SPECIAL_CHARACTER_CSB.to_vec(),
                _ => vec![*c],
            });
        }
        new_url
    }

    pub fn decode(to_decode: Vec<u8>) -> Vec<u8> {
        let mut new_url = Vec::new();
        let mut pos = 0;
        while pos < to_decode.len() {
            new_url.extend_from_slice(&[match to_decode[pos] {
                CHARACTER_PERCENTAGE => {
                    let special_character = &to_decode[pos..pos + 3];
                    pos += 3;
                    match special_character {
                        SPECIAL_CHARACTER_SPACE => CHARACTER_SPACE,
                        SPECIAL_CHARACTER_PERCENTAGE => CHARACTER_PERCENTAGE,
                        SPECIAL_CHARACTER_LOWER => CHARACTER_LOWER,
                        SPECIAL_CHARACTER_HIGHER => CHARACTER_HIGHER,
                        SPECIAL_CHARACTER_HASHTAG => CHARACTER_HASHTAG,
                        SPECIAL_CHARACTER_PIPE => CHARACTER_PIPE,
                        SPECIAL_CHARACTER_DQM => CHARACTER_DQM,
                        SPECIAL_CHARACTER_EM => CHARACTER_EM,
                        SPECIAL_CHARACTER_ASTERISTIC => CHARACTER_ASTERISTIC,
                        SPECIAL_CHARACTER_SQM => CHARACTER_SQM,
                        SPECIAL_CHARACTER_OP => CHARACTER_OP,
                        SPECIAL_CHARACTER_CP => CHARACTER_CP,
                        SPECIAL_CHARACTER_COLON => CHARACTER_COLON,
                        SPECIAL_CHARACTER_SEMICOLON => CHARACTER_SEMICOLON,
                        SPECIAL_CHARACTER_ATSIGN => CHARACTER_ATSIGN,
                        SPECIAL_CHARACTER_AMPERSAND => CHARACTER_AMPERSAND,
                        SPECIAL_CHARACTER_EQUAL => CHARACTER_EQUAL,
                        SPECIAL_CHARACTER_PLUS => CHARACTER_PLUS,
                        SPECIAL_CHARACTER_PS => CHARACTER_PS,
                        SPECIAL_CHARACTER_COMMA => CHARACTER_COMMA,
                        SPECIAL_CHARACTER_SLASH => CHARACTER_SLASH,
                        SPECIAL_CHARACTER_QM => CHARACTER_QM,
                        SPECIAL_CHARACTER_OSB => CHARACTER_OSB,
                        SPECIAL_CHARACTER_CSB => CHARACTER_CSB,
                        _ => to_decode[pos],
                    }
                }
                _ => {
                    pos += 1;
                    to_decode[pos - 1]
                }
            }]);
        }

        new_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::from_utf8;

    #[test]
    fn test_1_encode_url() {
        let url = b"https://www.twitter.com/search".to_vec();

        assert_eq!(
            from_utf8(&UrlEncoder::encode(url))
                .expect("Error in test-1: Unable to encode the url."),
            "https%3A%2F%2Fwww.twitter.com%2Fsearch"
        );
    }

    #[test]
    fn test_2_decode_url() {
        let url = b"https%3A%2F%2Fwww.twitter.com%2Fsearch".to_vec();

        assert_eq!(
            from_utf8(&UrlEncoder::decode(url))
                .expect("Error in test-2: Unable to decode the url."),
            "https://www.twitter.com/search"
        );
    }
}
