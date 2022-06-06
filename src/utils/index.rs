use rand::random;
use std::fmt::Write;

pub fn u8_to_hexa(bytes: &[u8]) -> String {
    let mut hexa = String::new();
    for byte in bytes {
        write!(&mut hexa, "{:x}", byte).expect("Unable to write byte to hexa");
    }

    hexa
}

pub fn random_u64_as_bytes() -> [u8; 8] {
    let mut vec = [0; 8];

    for x in vec.iter_mut() {
        *x = random()
    }

    vec
}

pub fn split_u8(source: Vec<u8>, split_seq: &[u8]) -> Vec<u8> {
    let mut pos = 0;

    while pos + split_seq.len() <= source.len() {
        let slice = &source[pos..pos + split_seq.len()];
        if slice.eq(split_seq) {
            break;
        };
        pos += 1;
    }

    source[pos + split_seq.len()..].to_vec()
}
