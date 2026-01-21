// Copyright 2026 driedpampas@proton.me
// SPDX-License-Identifier: GPL-3.0-or-later

use std::error::Error;
use std::fmt;

/// Error returned when Modified UTF-8 decoding fails.
#[derive(Debug, Clone)]
pub struct Mutf8Error(String);

impl fmt::Display for Mutf8Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MUTF-8 error: {}", self.0)
    }
}

impl Error for Mutf8Error {}

/// Decodes a Modified UTF-8 (MUTF-8) byte slice into a standard Rust `String`.
///
/// MUTF-8 is used by Minecraft (and Java) to represent strings. It differs from standard UTF-8
/// by encoding null characters as two bytes (`0xC0 0x80`) and encoding supplementary characters
/// as surrogate pairs using 6-byte sequences.
pub fn decode_mutf8(data: &[u8]) -> Result<String, Mutf8Error> {
    // Fast path: check if all bytes are ASCII (0x01..0x7F).
    // Standard UTF-8 and MUTF-8 are identical for ASCII-7 except for null (0x00).
    // In NBT, strings are often ASCII and don't contain nulls.
    if data.iter().all(|&b| b > 0 && b < 0x80) {
        // SAFETY: We just checked that all bytes are in the range 0x01..0x7F,
        // which is a valid UTF-8 sequence.
        return Ok(unsafe { String::from_utf8_unchecked(data.to_vec()) });
    }

    let mut result = String::with_capacity(data.len());
    let mut i = 0;

    while i < data.len() {
        let b = data[i];
        if b < 0x80 {
            // 1-byte (includes null handled as 0x00 if present, though MUTF-8 usually uses 0xC0 0x80)
            result.push(b as char);
            i += 1;
        } else if b & 0xE0 == 0xC0 {
            // 2-byte (includes 0xC0 0x80 for null)
            if i + 1 >= data.len() {
                return Err(Mutf8Error("Unexpected end of 2-byte sequence".to_string()));
            }
            let b2 = data[i + 1];
            if b2 & 0xC0 != 0x80 {
                return Err(Mutf8Error(
                    "Invalid continuation byte in 2-byte sequence".to_string(),
                ));
            }
            let val = ((b as u32 & 0x1F) << 6) | (b2 as u32 & 0x3F);
            result.push(
                char::from_u32(val).ok_or_else(|| Mutf8Error("Invalid character".to_string()))?,
            );
            i += 2;
        } else if b & 0xF0 == 0xE0 {
            // 3-byte
            if i + 2 >= data.len() {
                return Err(Mutf8Error("Unexpected end of 3-byte sequence".to_string()));
            }
            let b2 = data[i + 1];
            let b3 = data[i + 2];
            if b2 & 0xC0 != 0x80 || b3 & 0xC0 != 0x80 {
                return Err(Mutf8Error(
                    "Invalid continuation byte in 3-byte sequence".to_string(),
                ));
            }
            let val = ((b as u32 & 0x0F) << 12) | ((b2 as u32 & 0x3F) << 6) | (b3 as u32 & 0x3F);

            // Check for surrogate pairs in MUTF-8 (6-byte sequence)
            // A high surrogate starts with 0xED 0xA0..0xAF
            if (0xD800..=0xDBFF).contains(&val) && i + 5 < data.len() && data[i + 3] == 0xED {
                let b4 = data[i + 3]; // 0xED
                let b5 = data[i + 4];
                let b6 = data[i + 5];
                if b5 & 0xF0 == 0xB0 {
                    // 0xB0..0xBF
                    let val2 =
                        ((b4 as u32 & 0x0F) << 12) | ((b5 as u32 & 0x3F) << 6) | (b6 as u32 & 0x3F);
                    if (0xDC00..=0xDFFF).contains(&val2) {
                        // It IS a surrogate pair
                        let high = val - 0xD800;
                        let low = val2 - 0xDC00;
                        let codepoint = 0x10000 + ((high << 10) | low);
                        result.push(
                            char::from_u32(codepoint)
                                .ok_or_else(|| Mutf8Error("Invalid surrogate pair".to_string()))?,
                        );
                        i += 6;
                        continue;
                    }
                }
            }

            result.push(
                char::from_u32(val).ok_or_else(|| Mutf8Error("Invalid character".to_string()))?,
            );
            i += 3;
        } else {
            return Err(Mutf8Error(format!("Invalid byte 0x{:02X}", b)));
        }
    }

    Ok(result)
}

/// Encodes a standard Rust string into Modified UTF-8 (MUTF-8) bytes.
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
