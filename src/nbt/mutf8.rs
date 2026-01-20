use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Mutf8Error(String);

impl fmt::Display for Mutf8Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MUTF-8 error: {}", self.0)
    }
}

impl Error for Mutf8Error {}

pub fn decode_mutf8(data: &[u8]) -> Result<String, Mutf8Error> {
    let mut utf16 = Vec::new();
    let mut i = 0;

    while i < data.len() {
        let b = data[i];
        if b & 0x80 == 0 {
            // 1-byte
            utf16.push(b as u16);
            i += 1;
        } else if b & 0xE0 == 0xC0 {
            // 2-byte
            if i + 1 >= data.len() {
                return Err(Mutf8Error("Unexpected end of 2-byte sequence".to_string()));
            }
            let b2 = data[i + 1];
            let val = ((b as u16 & 0x1F) << 6) | (b2 as u16 & 0x3F);
            utf16.push(val);
            i += 2;
        } else if b & 0xF0 == 0xE0 {
            // 3-byte
            if i + 2 >= data.len() {
                return Err(Mutf8Error("Unexpected end of 3-byte sequence".to_string()));
            }
            let b2 = data[i + 1];
            let b3 = data[i + 2];
            let val = ((b as u16 & 0x0F) << 12) | ((b2 as u16 & 0x3F) << 6) | (b3 as u16 & 0x3F);
            utf16.push(val);
            i += 3;
        } else {
            return Err(Mutf8Error(format!("Invalid byte 0x{:02X}", b)));
        }
    }

    String::from_utf16(&utf16).map_err(|e| Mutf8Error(e.to_string()))
}

pub fn encode_mutf8(s: &str) -> Vec<u8> {
    let mut result = Vec::new();
    for c in s.encode_utf16() {
        if c == 0 {
            result.push(0xC0);
            result.push(0x80);
        } else if c < 0x80 {
            result.push(c as u8);
        } else if c < 0x800 {
            result.push(0xC0 | ((c >> 6) as u8));
            result.push(0x80 | ((c & 0x3F) as u8));
        } else {
            result.push(0xE0 | ((c >> 12) as u8));
            result.push(0x80 | (((c >> 6) & 0x3F) as u8));
            result.push(0x80 | ((c & 0x3F) as u8));
        }
    }
    result
}
