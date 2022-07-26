pub struct UrlEncoder;
pub use super::errors::UrlEncoderError;

impl UrlEncoder {
    pub fn encode_binary_data(to_encode: &[u8]) -> String {
        let mut new_url = Vec::new();
        for c in to_encode {
            new_url.extend_from_slice(&match *c {
                b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'-' | b'~' | b'.' | b'_' => vec![*c],
                _ => format!("%{:02X}", *c).bytes().collect(),
            });
        }
        String::from_utf8(new_url).expect("Failed to encode binary data")
    }

    pub fn decode_binary_data(to_decode: String) -> Result<Vec<u8>, UrlEncoderError> {
        let mut data: Vec<u8> = vec![];

        let check_first = to_decode.starts_with('%');
        let split: Vec<&str> = to_decode.split('%').collect();

        for (index, slice) in split.iter().enumerate() {
            let slice = slice.to_string();

            for i in 0..slice.len() {
                if check_first && index == 0 && i == 1 {
                    continue;
                }

                if index != 0 && i == 1 {
                    continue;
                }

                let byte = if i == 0 && (if index == 0 { check_first } else { true }) {
                    let maybe_hex: String = slice.chars().take(2).collect();

                    u8::from_str_radix(&maybe_hex, 16).or(Err(UrlEncoderError::FailedToEncode))?
                } else {
                    let maybe_hex: String = slice.chars().skip(i).take(1).collect();

                    let byte = maybe_hex.as_bytes();

                    byte[0]
                };

                data.push(byte);
            }
        }

        Ok(data)
    }
}
