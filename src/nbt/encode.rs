// Copyright 2026 driedpampas@proton.me
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::nbt::NbtTag;
use crate::nbt::mutf8::encode_mutf8;
use byteorder::{BigEndian, WriteBytesExt};
use std::io::{Result, Write};

/// Writes a length-prefixed Modified UTF-8 string to the writer.
pub fn write_nbt_string<W: Write>(writer: &mut W, s: &str) -> Result<()> {
    let bytes = encode_mutf8(s);
    writer.write_u16::<BigEndian>(bytes.len() as u16)?;
    writer.write_all(&bytes)?;
    Ok(())
}

/// Writes the payload of an NBT tag to the writer.
///
/// This does not include the type ID or the name of the tag.
pub fn write_tag_payload<W: Write>(writer: &mut W, tag: &NbtTag) -> Result<()> {
    match tag {
        NbtTag::End => Ok(()),
        NbtTag::Byte(v) => writer.write_i8(*v),
        NbtTag::Short(v) => writer.write_i16::<BigEndian>(*v),
        NbtTag::Int(v) => writer.write_i32::<BigEndian>(*v),
        NbtTag::Long(v) => writer.write_i64::<BigEndian>(*v),
        NbtTag::Float(v) => writer.write_f32::<BigEndian>(*v),
        NbtTag::Double(v) => writer.write_f64::<BigEndian>(*v),
        NbtTag::ByteArray(v) => {
            writer.write_i32::<BigEndian>(v.len() as i32)?;
            writer.write_all(v)
        }
        NbtTag::String(v) => write_nbt_string(writer, v),
        NbtTag::List(v) => {
            if v.is_empty() {
                writer.write_u8(0)?; // Tag_End as element type
                writer.write_i32::<BigEndian>(0)?;
            } else {
                let element_type = v[0].get_type_id();
                writer.write_u8(element_type)?;
                writer.write_i32::<BigEndian>(v.len() as i32)?;
                for element in v {
                    write_tag_payload(writer, element)?;
                }
            }
            Ok(())
        }
        NbtTag::Compound(v) => {
            for (name, tag) in v {
                writer.write_u8(tag.get_type_id())?;
                write_nbt_string(writer, name)?;
                write_tag_payload(writer, tag)?;
            }
            writer.write_u8(0)?; // Tag_End
            Ok(())
        }
        NbtTag::IntArray(v) => {
            writer.write_i32::<BigEndian>(v.len() as i32)?;
            for &i in v {
                writer.write_i32::<BigEndian>(i)?;
            }
            Ok(())
        }
        NbtTag::LongArray(v) => {
            writer.write_i32::<BigEndian>(v.len() as i32)?;
            for &i in v {
                writer.write_i64::<BigEndian>(i)?;
            }
            Ok(())
        }
    }
}

/// Writes a named tag (type ID + name + payload) to the writer.
///
/// This is the standard way to encode a root NBT tag for storage.
pub fn write_named_tag<W: Write>(writer: &mut W, name: &str, tag: &NbtTag) -> Result<()> {
    writer.write_u8(tag.get_type_id())?;
    write_nbt_string(writer, name)?;
    write_tag_payload(writer, tag)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_string() {
        let mut buf = Vec::new();
        write_nbt_string(&mut buf, "hi!").unwrap();
        assert_eq!(buf, vec![0, 3, b'h', b'i', b'!']);
    }

    #[test]
    fn test_round_trip_compound() {
        use indexmap::IndexMap;
        let mut map = IndexMap::new();
        map.insert("byte".to_string(), NbtTag::Byte(42));
        map.insert("string".to_string(), NbtTag::String("val".to_string()));
        let root = NbtTag::Compound(map);

        let mut buf = Vec::new();
        write_named_tag(&mut buf, "root", &root).unwrap();

        let mut input = &buf[..];
        let (name, decoded) =
            crate::nbt::parse::parse_named_tag::<nom::error::Error<&[u8]>>(&mut input).unwrap();

        assert_eq!(name, "root");
        assert_eq!(decoded, root);
    }
}
