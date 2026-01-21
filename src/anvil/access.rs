// Copyright 2026 driedpampas@proton.me
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::anvil::{ChunkLocation, CompressionType, RegionHeader, SECTOR_SIZE};
use crate::nbt::NbtTag;
use crate::nbt::parse::parse_named_tag;
use flate2::read::{GzDecoder, ZlibDecoder};
use memmap2::Mmap;
use std::fs::File;
use std::io::{Read, Result};
use std::path::Path;

/// A memory-mapped Anvil region file.
///
/// This struct provides efficient access to chunks within a `.mca` file.
pub struct Region {
    mmap: Mmap,
    header: RegionHeader,
}

impl Region {
    /// Opens an Anvil region file and memory-maps it.
    ///
    /// The headers are parsed immediately to allow quick lookups.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        if mmap.len() < SECTOR_SIZE * 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "MCA file too small for headers",
            ));
        }

        let mut locations = [ChunkLocation {
            offset: 0,
            sector_count: 0,
        }; 1024];
        let mut timestamps = [0u32; 1024];

        for (i, location) in locations.iter_mut().enumerate() {
            let start = i * 4;
            let offset = ((mmap[start] as u32) << 16)
                | ((mmap[start + 1] as u32) << 8)
                | (mmap[start + 2] as u32);
            let sector_count = mmap[start + 3];
            *location = ChunkLocation {
                offset,
                sector_count,
            };
        }

        for (i, timestamp_slot) in timestamps.iter_mut().enumerate() {
            let start = SECTOR_SIZE + i * 4;
            let timestamp = ((mmap[start] as u32) << 24)
                | ((mmap[start + 1] as u32) << 16)
                | ((mmap[start + 2] as u32) << 8)
                | (mmap[start + 3] as u32);
            *timestamp_slot = timestamp;
        }

        Ok(Region {
            mmap,
            header: RegionHeader {
                locations,
                timestamps,
            },
        })
    }

    /// Retrieves the raw decompressed NBT data for a chunk at the given world coordinates.
    ///
    /// Coordinates are in chunk units (not blocks). For example, (0, 0) is the first chunk
    /// in the region, and (31, 31) is the last. Coordinates outside the 0-31 range will
    /// be wrapped using `rem_euclid(32)`.
    ///
    /// Returns `Ok(Some(data))` if the chunk exists and was successfully decompressed,
    /// `Ok(None)` if the chunk is not present in this region file, or an `Err` if
    /// decompression fails or the file is corrupted.
    pub fn get_chunk_data(&self, x: i32, z: i32) -> Result<Option<Vec<u8>>> {
        let rel_x = x.rem_euclid(32);
        let rel_z = z.rem_euclid(32);
        let index = (rel_z * 32 + rel_x) as usize;

        let location = self.header.locations[index];
        if location.offset == 0 {
            return Ok(None);
        }

        let start_byte = location.offset as usize * SECTOR_SIZE;
        let length = ((self.mmap[start_byte] as u32) << 24)
            | ((self.mmap[start_byte + 1] as u32) << 16)
            | ((self.mmap[start_byte + 2] as u32) << 8)
            | (self.mmap[start_byte + 3] as u32);

        if length < 1 {
            return Ok(None);
        }

        let compression_type_raw = self.mmap[start_byte + 4];
        let compression_type = CompressionType::try_from(compression_type_raw)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let data = &self.mmap[start_byte + 5..start_byte + 4 + length as usize];

        let mut decoded = Vec::new();
        match compression_type {
            CompressionType::Gzip => {
                let mut decoder = GzDecoder::new(data);
                decoder.read_to_end(&mut decoded)?;
            }
            CompressionType::Zlib => {
                let mut decoder = ZlibDecoder::new(data);
                decoder.read_to_end(&mut decoded)?;
            }
            CompressionType::None => {
                decoded.extend_from_slice(data);
            }
        }

        Ok(Some(decoded))
    }

    /// Parses the NBT data for a chunk at the given world coordinates.
    ///
    /// This is a convenience method that calls [`get_chunk_data`](Self::get_chunk_data)
    /// and then parses the resulting bytes into an [`NbtTag`].
    pub fn get_chunk_nbt(&self, x: i32, z: i32) -> Result<Option<(String, NbtTag)>> {
        if let Some(data) = self.get_chunk_data(x, z)? {
            let mut input = &data[..];
            let result = parse_named_tag(&mut input).map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Failed to parse NBT")
            })?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}
