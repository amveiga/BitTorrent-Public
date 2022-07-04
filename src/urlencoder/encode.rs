pub struct UrlEncoder;

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
}
