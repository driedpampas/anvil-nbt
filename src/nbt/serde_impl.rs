// Copyright 2026 driedpampas@proton.me
// SPDX-License-Identifier: GPL-3.0-or-later

//! Serde support for NBT.
//!
//! This module provides functions to convert between Rust types and [`NbtTag`].
//! It requires the `serde` feature to be enabled.

#![cfg_attr(docsrs, doc(cfg(feature = "serde")))]

use crate::nbt::NbtTag;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize, de, ser};
use std::fmt;
use thiserror::Error;

/// Errors that can occur during NBT serde operations.
#[derive(Debug, Error)]
pub enum SerdeError {
    /// A custom error from a `Serialize` or `Deserialize` implementation.
    #[error("Message: {0}")]
    Custom(String),
    /// The type is not supported by NBT (e.g., maps with non-string keys).
    #[error("Unsupported type for NBT")]
    UnsupportedType,
    /// Expected a compound tag during deserialization.
    #[error("Expected a compound tag")]
    ExpectedCompound,
    /// A required field was missing during deserialization.
    #[error("Missing field: {0}")]
    MissingField(String),
}

impl ser::Error for SerdeError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        SerdeError::Custom(msg.to_string())
    }
}

impl de::Error for SerdeError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        SerdeError::Custom(msg.to_string())
    }
}

/// Converts a type that implements [`Serialize`] to an [`NbtTag`].
///
/// # Errors
///
/// Returns a [`SerdeError`] if the type cannot be represented as NBT.
pub fn to_nbt<T: Serialize>(value: &T) -> Result<NbtTag, SerdeError> {
    value.serialize(NbtSerializer)
}

/// Converts an [`NbtTag`] to a type that implements [`Deserialize`].
///
/// # Errors
///
/// Returns a [`SerdeError`] if the NBT data does not match the expected structure of `T`.
pub fn from_nbt<'a, T: Deserialize<'a>>(tag: NbtTag) -> Result<T, SerdeError> {
    T::deserialize(NbtDeserializer::new(tag))
}

/// Internal serializer for converting Rust types to [`NbtTag`].
struct NbtSerializer;

impl ser::Serializer for NbtSerializer {
    type Ok = NbtTag;
    type Error = SerdeError;

    type SerializeSeq = SerializeSeq;
    type SerializeTuple = SerializeSeq;
    type SerializeTupleStruct = SerializeSeq;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeMap;
    type SerializeStructVariant = SerializeStructVariant;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(NbtTag::Byte(if v { 1 } else { 0 }))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(NbtTag::Byte(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(NbtTag::Short(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(NbtTag::Int(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(NbtTag::Long(v))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(NbtTag::Byte(v as i8))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(NbtTag::Short(v as i16))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(NbtTag::Int(v as i32))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(NbtTag::Long(v as i64))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(NbtTag::Float(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(NbtTag::Double(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(NbtTag::String(v.to_owned()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(NbtTag::ByteArray(v.to_vec()))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(NbtTag::End) // Representing None as End is a choice, might change based on context
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(NbtTag::End)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        let mut map = IndexMap::new();
        map.insert(variant.to_owned(), value.serialize(self)?);
        Ok(NbtTag::Compound(map))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SerializeSeq {
            elements: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(SerializeTupleVariant {
            variant: variant.to_owned(),
            elements: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(SerializeMap {
            map: IndexMap::new(),
            next_key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(SerializeStructVariant {
            variant: variant.to_owned(),
            map: IndexMap::new(),
        })
    }
}

struct SerializeSeq {
    elements: Vec<NbtTag>,
}

impl ser::SerializeSeq for SerializeSeq {
    type Ok = NbtTag;
    type Error = SerdeError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.elements.push(value.serialize(NbtSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(NbtTag::List(self.elements))
    }
}

impl ser::SerializeTuple for SerializeSeq {
    type Ok = NbtTag;
    type Error = SerdeError;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        ser::SerializeSeq::serialize_element(self, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleStruct for SerializeSeq {
    type Ok = NbtTag;
    type Error = SerdeError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        ser::SerializeSeq::serialize_element(self, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

struct SerializeTupleVariant {
    variant: String,
    elements: Vec<NbtTag>,
}

impl ser::SerializeTupleVariant for SerializeTupleVariant {
    type Ok = NbtTag;
    type Error = SerdeError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.elements.push(value.serialize(NbtSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let mut map = IndexMap::new();
        map.insert(self.variant, NbtTag::List(self.elements));
        Ok(NbtTag::Compound(map))
    }
}

struct SerializeMap {
    map: IndexMap<String, NbtTag>,
    next_key: Option<String>,
}

impl ser::SerializeMap for SerializeMap {
    type Ok = NbtTag;
    type Error = SerdeError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Self::Error> {
        let key_nbt = key.serialize(NbtSerializer)?;
        if let NbtTag::String(s) = key_nbt {
            self.next_key = Some(s);
            Ok(())
        } else {
            Err(ser::Error::custom("NBT map keys must be strings"))
        }
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        let key = self.next_key.take().unwrap();
        self.map.insert(key, value.serialize(NbtSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(NbtTag::Compound(self.map))
    }
}

impl ser::SerializeStruct for SerializeMap {
    type Ok = NbtTag;
    type Error = SerdeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.map
            .insert(key.to_owned(), value.serialize(NbtSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(NbtTag::Compound(self.map))
    }
}

struct SerializeStructVariant {
    variant: String,
    map: IndexMap<String, NbtTag>,
}

impl ser::SerializeStructVariant for SerializeStructVariant {
    type Ok = NbtTag;
    type Error = SerdeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.map
            .insert(key.to_owned(), value.serialize(NbtSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let mut outer = IndexMap::new();
        outer.insert(self.variant, NbtTag::Compound(self.map));
        Ok(NbtTag::Compound(outer))
    }
}

/// Internal deserializer for converting [`NbtTag`] to Rust types.
struct NbtDeserializer {
    tag: NbtTag,
}

impl NbtDeserializer {
    fn new(tag: NbtTag) -> Self {
        NbtDeserializer { tag }
    }
}

impl<'de> de::Deserializer<'de> for NbtDeserializer {
    type Error = SerdeError;

    fn deserialize_any<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.tag {
            NbtTag::End => visitor.visit_unit(),
            NbtTag::Byte(v) => visitor.visit_i8(v),
            NbtTag::Short(v) => visitor.visit_i16(v),
            NbtTag::Int(v) => visitor.visit_i32(v),
            NbtTag::Long(v) => visitor.visit_i64(v),
            NbtTag::Float(v) => visitor.visit_f32(v),
            NbtTag::Double(v) => visitor.visit_f64(v),
            NbtTag::ByteArray(v) => visitor.visit_byte_buf(v),
            NbtTag::String(v) => visitor.visit_string(v),
            NbtTag::List(v) => visitor.visit_seq(SeqAccess {
                iter: v.into_iter(),
            }),
            NbtTag::Compound(v) => visitor.visit_map(MapAccess {
                iter: v.into_iter(),
                next_value: None,
            }),
            NbtTag::IntArray(v) => visitor.visit_seq(SeqAccess {
                iter: v
                    .into_iter()
                    .map(NbtTag::Int)
                    .collect::<Vec<_>>()
                    .into_iter(),
            }),
            NbtTag::LongArray(v) => visitor.visit_seq(SeqAccess {
                iter: v
                    .into_iter()
                    .map(NbtTag::Long)
                    .collect::<Vec<_>>()
                    .into_iter(),
            }),
        }
    }

    fn deserialize_bool<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.tag {
            NbtTag::Byte(v) => visitor.visit_bool(v != 0),
            _ => self.deserialize_any(visitor),
        }
    }

    fn deserialize_option<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.tag {
            NbtTag::End => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_newtype_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        match self.tag {
            NbtTag::String(s) => visitor.visit_enum(EnumAccess {
                variant: s,
                value: None,
            }),
            NbtTag::Compound(m) => {
                if m.len() == 1 {
                    let (k, v) = m.into_iter().next().unwrap();
                    visitor.visit_enum(EnumAccess {
                        variant: k,
                        value: Some(v),
                    })
                } else {
                    Err(de::Error::custom(
                        "Expected compound with single key for enum",
                    ))
                }
            }
            _ => Err(de::Error::custom("Expected string or compound for enum")),
        }
    }

    serde::forward_to_deserialize_any! {
        i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }
}

struct SeqAccess {
    iter: std::vec::IntoIter<NbtTag>,
}

impl<'de> de::SeqAccess<'de> for SeqAccess {
    type Error = SerdeError;

    fn next_element_seed<T: de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Self::Error> {
        match self.iter.next() {
            Some(tag) => seed.deserialize(NbtDeserializer::new(tag)).map(Some),
            None => Ok(None),
        }
    }
}

struct MapAccess {
    iter: indexmap::map::IntoIter<String, NbtTag>,
    next_value: Option<NbtTag>,
}

impl<'de> de::MapAccess<'de> for MapAccess {
    type Error = SerdeError;

    fn next_key_seed<K: de::DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        match self.iter.next() {
            Some((k, v)) => {
                self.next_value = Some(v);
                seed.deserialize(de::value::StringDeserializer::new(k))
                    .map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V: de::DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        let v = self.next_value.take().unwrap();
        seed.deserialize(NbtDeserializer::new(v))
    }
}

struct EnumAccess {
    variant: String,
    value: Option<NbtTag>,
}

impl<'de> de::EnumAccess<'de> for EnumAccess {
    type Error = SerdeError;
    type Variant = VariantAccess;

    fn variant_seed<V: de::DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Self::Error> {
        let variant = seed.deserialize(de::value::StringDeserializer::new(self.variant))?;
        Ok((variant, VariantAccess { value: self.value }))
    }
}

struct VariantAccess {
    value: Option<NbtTag>,
}

impl<'de> de::VariantAccess<'de> for VariantAccess {
    type Error = SerdeError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        match self.value {
            Some(_) => Err(de::Error::custom("Expected unit variant")),
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T: de::DeserializeSeed<'de>>(
        self,
        seed: T,
    ) -> Result<T::Value, Self::Error> {
        match self.value {
            Some(tag) => seed.deserialize(NbtDeserializer::new(tag)),
            None => Err(de::Error::custom("Expected newtype variant")),
        }
    }

    fn tuple_variant<V: de::Visitor<'de>>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        match self.value {
            Some(NbtTag::List(v)) => visitor.visit_seq(SeqAccess {
                iter: v.into_iter(),
            }),
            _ => Err(de::Error::custom("Expected list for tuple variant")),
        }
    }

    fn struct_variant<V: de::Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        match self.value {
            Some(NbtTag::Compound(v)) => visitor.visit_map(MapAccess {
                iter: v.into_iter(),
                next_value: None,
            }),
            _ => Err(de::Error::custom("Expected compound for struct variant")),
        }
    }
}
