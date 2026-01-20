// Copyright 2026 driedpampas@proton.me
// SPDX-License-Identifier: GPL-3.0-or-later

//! Core NBT data structures and types.

pub mod encode;
pub mod mutf8;
pub mod parse;

use indexmap::IndexMap;

/// Represents a Minecraft NBT (Named Binary Tag).
///
/// NBT is a tree-based storage format used by Minecraft for player data, level data, and chunks.
/// This enum covers all possible tag types in the format.
#[derive(Debug, Clone, PartialEq)]
pub enum NbtTag {
    /// Marker tag used to signify the end of a `Compound` tag.
    End,
    /// A single signed byte.
    Byte(i8),
    /// A 16-bit signed integer.
    Short(i16),
    /// A 32-bit signed integer.
    Int(i32),
    /// A 64-bit signed integer.
    Long(i64),
    /// A 32-bit floating point number.
    Float(f32),
    /// A 64-bit floating point number.
    Double(f64),
    /// An array of bytes.
    ByteArray(Vec<u8>),
    /// A UTF-8 string (encoded as Modified UTF-8 on disk).
    String(String),
    /// A list of tags of the same type.
    List(Vec<NbtTag>),
    /// A map of named tags. Uses `IndexMap` to preserve field order.
    Compound(IndexMap<String, NbtTag>),
    /// An array of 32-bit signed integers.
    IntArray(Vec<i32>),
    /// An array of 64-bit signed integers.
    LongArray(Vec<i64>),
}

impl NbtTag {
    pub fn get_type_id(&self) -> u8 {
        match self {
            NbtTag::End => 0,
            NbtTag::Byte(_) => 1,
            NbtTag::Short(_) => 2,
            NbtTag::Int(_) => 3,
            NbtTag::Long(_) => 4,
            NbtTag::Float(_) => 5,
            NbtTag::Double(_) => 6,
            NbtTag::ByteArray(_) => 7,
            NbtTag::String(_) => 8,
            NbtTag::List(_) => 9,
            NbtTag::Compound(_) => 10,
            NbtTag::IntArray(_) => 11,
            NbtTag::LongArray(_) => 12,
        }
    }
}
