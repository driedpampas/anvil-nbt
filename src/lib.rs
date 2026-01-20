//! # mcse-nbt
//!
//! A production-grade Rust library for parsing and encoding Minecraft's NBT and Anvil (.mca) formats.
//!
//! This library provides efficient, safe, and bit-perfect handling of Minecraft world data.
//! Key features include:
//! - Full NBT support (including Modified UTF-8)
//! - Lazy-loading Anvil region files with memory mapping
//! - High-performance parsing using `nom`
//! - Idempotent round-trips for both NBT and Anvil data

pub mod anvil;
pub mod nbt;
