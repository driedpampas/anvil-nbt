// Copyright 2026 driedpampas@proton.me
// SPDX-License-Identifier: GPL-3.0-or-later

#![cfg_attr(docsrs, feature(doc_cfg))]

//! # anvil-nbt
//!
//! A Rust library for parsing and encoding Minecraft's NBT and Anvil (.mca) formats.
//!
//! This library provides efficient, safe, and bit-perfect handling of Minecraft world data.
//! Key features include:
//! - Full NBT support (including Modified UTF-8)
//! - Optional `serde` support for serializing Rust types to NBT (via `serde` feature)
//! - Lazy-loading Anvil region files with memory mapping
//! - High-performance manual parsing (removed `nom` overhead)
//! - Idempotent round-trips for both NBT and Anvil data

pub mod anvil;
pub mod nbt;
