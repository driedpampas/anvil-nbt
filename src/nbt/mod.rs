// Copyright 2026 driedpampas@proton.me
// SPDX-License-Identifier: GPL-3.0-or-later

//! Core NBT data structures and types.

pub mod encode;
pub mod mutf8;
pub mod parse;
#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub mod serde_impl;

use indexmap::IndexMap;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Represents a Minecraft NBT (Named Binary Tag).
///
/// NBT is a tree-based storage format used by Minecraft for player data, level data, and chunks.
/// This enum covers all possible tag types in the format.
///
/// # Examples
///
/// ```
/// use anvil_nbt::nbt::NbtTag;
/// let tag = NbtTag::Int(42);
/// assert_eq!(tag.get_type_id(), 3);
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum NbtTag {
    /// Marker tag used to signify the end of a `Compound` tag. (ID: 0)
    End,
    /// A single signed byte. (ID: 1)
    Byte(i8),
    /// A 16-bit signed integer. (ID: 2)
    Short(i16),
    /// A 32-bit signed integer. (ID: 3)
    Int(i32),
    /// A 64-bit signed integer. (ID: 4)
    Long(i64),
    /// A 32-bit floating point number. (ID: 5)
    Float(f32),
    /// A 64-bit floating point number. (ID: 6)
    Double(f64),
    /// An array of bytes. (ID: 7)
    ByteArray(Vec<u8>),
    /// A UTF-8 string (encoded as Modified UTF-8 on disk). (ID: 8)
    String(String),
    /// A list of tags of the same type. (ID: 9)
    List(Vec<NbtTag>),
    /// A map of named tags. Uses `IndexMap` to preserve field order. (ID: 10)
    Compound(IndexMap<String, NbtTag>),
    /// An array of 32-bit signed integers. (ID: 11)
    IntArray(Vec<i32>),
    /// An array of 64-bit signed integers. (ID: 12)
    LongArray(Vec<i64>),
}

impl NbtTag {
    /// Returns the type ID of the NBT tag according to the specification.
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
