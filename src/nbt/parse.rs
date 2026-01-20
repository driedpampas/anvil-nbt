use crate::nbt::NbtTag;
use crate::nbt::mutf8::decode_mutf8;
use indexmap::IndexMap;
use nom::{
    bytes::complete::take,
    error::ParseError,
    number::complete::{be_f32, be_f64, be_i8, be_i16, be_i32, be_i64, be_u8, be_u16},
};

/// Parses a length-prefixed Modified UTF-8 string from the input.
///
/// This is the standard string format used in NBT files.
pub fn parse_nbt_string<'a, E>(input: &mut &'a [u8]) -> Result<String, E>
where
    E: ParseError<&'a [u8]>,
{
    let (remaining, len) = be_u16(*input).map_err(unwrap_err)?;
    *input = remaining;
    let (remaining, bytes) = take(len as usize)(*input).map_err(unwrap_err)?;
    *input = remaining;
    decode_mutf8(bytes).map_err(|_| E::from_error_kind(*input, nom::error::ErrorKind::Tag))
}

fn unwrap_err<E>(e: nom::Err<E>) -> E {
    match e {
        nom::Err::Error(e) | nom::Err::Failure(e) => e,
        nom::Err::Incomplete(_) => panic!("Incomplete data"), // Or handle appropriately
    }
}

/// Parses the payload of an NBT tag based on its type ID.
///
/// This function is used recursively to parse lists and compounds.
pub fn parse_tag_payload<'a, E>(input: &mut &'a [u8], type_id: u8) -> Result<NbtTag, E>
where
    E: ParseError<&'a [u8]>,
{
    match type_id {
        0 => Ok(NbtTag::End),
        1 => {
            let (i, v) = be_i8(*input).map_err(unwrap_err)?;
            *input = i;
            Ok(NbtTag::Byte(v))
        }
        2 => {
            let (i, v) = be_i16(*input).map_err(unwrap_err)?;
            *input = i;
            Ok(NbtTag::Short(v))
        }
        3 => {
            let (i, v) = be_i32(*input).map_err(unwrap_err)?;
            *input = i;
            Ok(NbtTag::Int(v))
        }
        4 => {
            let (i, v) = be_i64(*input).map_err(unwrap_err)?;
            *input = i;
            Ok(NbtTag::Long(v))
        }
        5 => {
            let (i, v) = be_f32(*input).map_err(unwrap_err)?;
            *input = i;
            Ok(NbtTag::Float(v))
        }
        6 => {
            let (i, v) = be_f64(*input).map_err(unwrap_err)?;
            *input = i;
            Ok(NbtTag::Double(v))
        }
        7 => {
            let (i, len) = be_i32(*input).map_err(unwrap_err)?;
            *input = i;
            let (i, bytes) = take(len as usize)(*input).map_err(unwrap_err)?;
            *input = i;
            Ok(NbtTag::ByteArray(bytes.to_vec()))
        }
        8 => Ok(NbtTag::String(parse_nbt_string(input)?)),
        9 => {
            let (i, element_type) = be_u8(*input).map_err(unwrap_err)?;
            *input = i;
            let (i, len) = be_i32(*input).map_err(unwrap_err)?;
            *input = i;
            let mut elements = Vec::with_capacity(len as usize);
            for _ in 0..len {
                elements.push(parse_tag_payload(input, element_type)?);
            }
            Ok(NbtTag::List(elements))
        }
        10 => {
            let mut map = IndexMap::new();
            loop {
                let (i, tag_type) = be_u8(*input).map_err(unwrap_err)?;
                *input = i;
                if tag_type == 0 {
                    break;
                }
                let name = parse_nbt_string(input)?;
                let payload = parse_tag_payload(input, tag_type)?;
                map.insert(name, payload);
            }
            Ok(NbtTag::Compound(map))
        }
        11 => {
            let (i, len) = be_i32(*input).map_err(unwrap_err)?;
            *input = i;
            let mut ints = Vec::with_capacity(len as usize);
            for _ in 0..len {
                let (i, v) = be_i32(*input).map_err(unwrap_err)?;
                *input = i;
                ints.push(v);
            }
            Ok(NbtTag::IntArray(ints))
        }
        12 => {
            let (i, len) = be_i32(*input).map_err(unwrap_err)?;
            *input = i;
            let mut longs = Vec::with_capacity(len as usize);
            for _ in 0..len {
                let (i, v) = be_i64(*input).map_err(unwrap_err)?;
                *input = i;
                longs.push(v);
            }
            Ok(NbtTag::LongArray(longs))
        }
        _ => Err(E::from_error_kind(*input, nom::error::ErrorKind::Switch)),
    }
}

/// Parses a named tag (type ID + name + payload) from the input.
///
/// This is typically the entry point for parsing an uncompressed NBT file.
pub fn parse_named_tag<'a, E>(input: &mut &'a [u8]) -> Result<(String, NbtTag), E>
where
    E: ParseError<&'a [u8]>,
{
    let (i, tag_type) = be_u8(*input).map_err(unwrap_err)?;
    *input = i;
    if tag_type == 0 {
        return Ok(("".to_string(), NbtTag::End));
    }
    let name = parse_nbt_string(input)?;
    let payload = parse_tag_payload(input, tag_type)?;
    Ok((name, payload))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::error::Error;

    #[test]
    fn test_parse_string() {
        let data = vec![0, 3, b'h', b'i', b'!'];
        let mut input = &data[..];
        let s = parse_nbt_string::<Error<&[u8]>>(&mut input).unwrap();
        assert_eq!(s, "hi!");
        assert!(input.is_empty());
    }

    #[test]
    fn test_parse_byte() {
        let data = vec![42];
        let mut input = &data[..];
        let tag = parse_tag_payload::<Error<&[u8]>>(&mut input, 1).unwrap();
        if let NbtTag::Byte(v) = tag {
            assert_eq!(v, 42);
        } else {
            panic!("Wrong tag type");
        }
    }
}
