// Copyright 2026 driedpampas@proton.me
// SPDX-License-Identifier: GPL-3.0-or-later

//! Anvil region file format handling.

pub mod access;
pub mod encode;

/// The size of a single sector in an Anvil region file (4096 bytes).
pub const SECTOR_SIZE: usize = 4096;

/// Represents the location of a chunk within a region file.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChunkLocation {
    /// The offset of the chunk data in sectors from the start of the file.
    pub offset: u32,
    /// The number of sectors allocated for this chunk.
    pub sector_count: u8,
}

/// The header of a region file, containing locations and timestamps for all 1024 chunks.
#[derive(Debug, Clone)]
pub struct RegionHeader {
    /// Locations for chunks at (0,0) to (31,31).
    pub locations: [ChunkLocation; 1024],
    /// Last modification timestamps for chunks.
    pub timestamps: [u32; 1024],
}

/// Supported compression types for chunk data.
pub enum CompressionType {
    /// Gzip compression (standard for .dat files, less common in .mca).
    Gzip = 1,
    /// Zlib compression (standard for .mca chunks).
    Zlib = 2,
    /// No compression.
    None = 3,
}

impl TryFrom<u8> for CompressionType {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(CompressionType::Gzip),
            2 => Ok(CompressionType::Zlib),
            3 => Ok(CompressionType::None),
            _ => Err(format!("Unknown compression type: {}", value)),
        }
    }
}
