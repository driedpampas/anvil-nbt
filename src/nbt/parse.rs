// Copyright 2026 driedpampas@proton.me
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::nbt::NbtTag;
use crate::nbt::mutf8::decode_mutf8;
use indexmap::IndexMap;
/// A reader that maintains a cursor over a byte slice for manual parsing.
pub struct ByteReader<'a> {
    /// The remaining data to be read.
    pub data: &'a [u8],
}

/// Errors that can occur during NBT parsing.
#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    /// The input ended unexpectedly before a tag or field could be fully read.
    UnexpectedEof,
    /// An unknown or invalid NBT tag type ID was encountered.
    InvalidTag(u8),
    /// A string field could not be decoded as Modified UTF-8.
    InvalidString,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedEof => write!(f, "Unexpected EOF"),
            ParseError::InvalidTag(t) => write!(f, "Invalid tag type: {}", t),
            ParseError::InvalidString => write!(f, "Invalid MUTF-8 string"),
        }
    }
}

impl std::error::Error for ParseError {}

impl<'a> ByteReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    #[inline]
    fn read_u8(&mut self) -> Result<u8, ParseError> {
        if self.data.is_empty() {
            return Err(ParseError::UnexpectedEof);
        }
        let b = self.data[0];
        self.data = &self.data[1..];
        Ok(b)
    }

    #[inline]
    fn read_i8(&mut self) -> Result<i8, ParseError> {
        self.read_u8().map(|b| b as i8)
    }

    #[inline]
    fn read_u16(&mut self) -> Result<u16, ParseError> {
        if self.data.len() < 2 {
            return Err(ParseError::UnexpectedEof);
        }
        let bytes = [self.data[0], self.data[1]];
        self.data = &self.data[2..];
        Ok(u16::from_be_bytes(bytes))
    }

    #[inline]
    fn read_i16(&mut self) -> Result<i16, ParseError> {
        self.read_u16().map(|v| v as i16)
    }

    #[inline]
    fn read_i32(&mut self) -> Result<i32, ParseError> {
        if self.data.len() < 4 {
            return Err(ParseError::UnexpectedEof);
        }
        let bytes = [self.data[0], self.data[1], self.data[2], self.data[3]];
        self.data = &self.data[4..];
        Ok(i32::from_be_bytes(bytes))
    }

    #[inline]
    fn read_i64(&mut self) -> Result<i64, ParseError> {
        if self.data.len() < 8 {
            return Err(ParseError::UnexpectedEof);
        }
        let bytes: [u8; 8] = self.data[..8].try_into().unwrap();
        self.data = &self.data[8..];
        Ok(i64::from_be_bytes(bytes))
    }

    #[inline]
    fn read_f32(&mut self) -> Result<f32, ParseError> {
        self.read_i32().map(|v| f32::from_bits(v as u32))
    }

    #[inline]
    fn read_f64(&mut self) -> Result<f64, ParseError> {
        self.read_i64().map(|v| f64::from_bits(v as u64))
    }

    #[inline]
    fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], ParseError> {
        if self.data.len() < len {
            return Err(ParseError::UnexpectedEof);
        }
        let bytes = &self.data[..len];
        self.data = &self.data[len..];
        Ok(bytes)
    }
}

/// Parses a length-prefixed Modified UTF-8 string from the input.
pub fn parse_nbt_string(reader: &mut ByteReader) -> Result<String, ParseError> {
    let len = reader.read_u16()? as usize;
    let bytes = reader.read_bytes(len)?;
    decode_mutf8(bytes).map_err(|_| ParseError::InvalidString)
}

/// Parses the payload of an NBT tag based on its type ID.
pub fn parse_tag_payload(reader: &mut ByteReader, type_id: u8) -> Result<NbtTag, ParseError> {
    match type_id {
        0 => Ok(NbtTag::End),
        1 => Ok(NbtTag::Byte(reader.read_i8()?)),
        2 => Ok(NbtTag::Short(reader.read_i16()?)),
        3 => Ok(NbtTag::Int(reader.read_i32()?)),
        4 => Ok(NbtTag::Long(reader.read_i64()?)),
        5 => Ok(NbtTag::Float(reader.read_f32()?)),
        6 => Ok(NbtTag::Double(reader.read_f64()?)),
        7 => {
            let len = reader.read_i32()? as usize;
            let bytes = reader.read_bytes(len)?;
            Ok(NbtTag::ByteArray(bytes.to_vec()))
        }
        8 => Ok(NbtTag::String(parse_nbt_string(reader)?)),
        9 => {
            let element_type = reader.read_u8()?;
            let len = reader.read_i32()? as usize;
            let mut elements = Vec::with_capacity(len);
            for _ in 0..len {
                elements.push(parse_tag_payload(reader, element_type)?);
            }
            Ok(NbtTag::List(elements))
        }
        10 => {
            let mut map = IndexMap::new();
            loop {
                let tag_type = reader.read_u8()?;
                if tag_type == 0 {
                    break;
                }
                let name = parse_nbt_string(reader)?;
                let payload = parse_tag_payload(reader, tag_type)?;
                map.insert(name, payload);
            }
            Ok(NbtTag::Compound(map))
        }
        11 => {
            let len = reader.read_i32()? as usize;
            let byte_len = len * 4;
            let bytes = reader.read_bytes(byte_len)?;
            let mut ints = Vec::with_capacity(len);
            for chunk in bytes.chunks_exact(4) {
                ints.push(i32::from_be_bytes(chunk.try_into().unwrap()));
            }
            Ok(NbtTag::IntArray(ints))
        }
        12 => {
            let len = reader.read_i32()? as usize;
            let byte_len = len * 8;
            let bytes = reader.read_bytes(byte_len)?;
            let mut longs = Vec::with_capacity(len);
            for chunk in bytes.chunks_exact(8) {
                longs.push(i64::from_be_bytes(chunk.try_into().unwrap()));
            }
            Ok(NbtTag::LongArray(longs))
        }
        _ => Err(ParseError::InvalidTag(type_id)),
    }
}

/// Parses a named tag (type ID + name + payload) from the input.
///
/// This is the entry point for parsing top-level NBT data (like `level.dat`).
/// On success, returns the name of the tag and the tag itself, and updates `input`
/// to point to the remaining bytes.
pub fn parse_named_tag(input: &mut &[u8]) -> Result<(String, NbtTag), ParseError> {
    let mut reader = ByteReader::new(input);
    let tag_type = match reader.read_u8() {
        Ok(t) => t,
        Err(_) => return Err(ParseError::UnexpectedEof),
    };
    if tag_type == 0 {
        *input = reader.data;
        return Ok(("".to_string(), NbtTag::End));
    }
    let name = parse_nbt_string(&mut reader)?;
    let payload = parse_tag_payload(&mut reader, tag_type)?;
    *input = reader.data;
    Ok((name, payload))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_string() {
        let data = vec![0, 3, b'h', b'i', b'!'];
        let mut reader = ByteReader::new(&data);
        let s = parse_nbt_string(&mut reader).unwrap();
        assert_eq!(s, "hi!");
        assert!(reader.data.is_empty());
    }

    #[test]
    fn test_parse_byte() {
        let data = vec![42];
        let mut reader = ByteReader::new(&data);
        let tag = parse_tag_payload(&mut reader, 1).unwrap();
        if let NbtTag::Byte(v) = tag {
            assert_eq!(v, 42);
        } else {
            panic!("Wrong tag type");
        }
    }
}
