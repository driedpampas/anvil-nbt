// Copyright 2026 driedpampas@proton.me
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::anvil::{ChunkLocation, SECTOR_SIZE};
use crate::nbt::NbtTag;
use crate::nbt::encode::write_named_tag;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::io::{Result, Seek, SeekFrom, Write};

/// A writer for creating or modifying Anvil region files.
#[allow(dead_code)]
pub struct RegionWriter<W: Write + Seek> {
    #[allow(dead_code)]
    writer: W,
}

impl<W: Write + Seek> RegionWriter<W> {
    /// Creates a new `RegionWriter` wrapping the given writer.
    pub fn new(writer: W) -> Self {
        RegionWriter { writer }
    }

    /// Writes all provided chunks to the region file.
    ///
    /// This method encodes and compresses each chunk using Zlib,
    /// then writes them to the underlying writer along with the required headers.
    pub fn write_all_chunks(&mut self, chunks: &[(i32, i32, String, NbtTag)]) -> Result<()> {
        let mut locations = [ChunkLocation {
            offset: 0,
            sector_count: 0,
        }; 1024];

        // Move past header space (4096 bytes for locations + 4096 bytes for timestamps)
        self.writer.seek(SeekFrom::Start(SECTOR_SIZE as u64 * 2))?;
        let mut current_sector = 2u32;

        for (x, z, name, tag) in chunks {
            let rel_x = (x % 32 + 32) % 32;
            let rel_z = (z % 32 + 32) % 32;
            let index = (rel_z * 32 + rel_x) as usize;

            // Encode and compress chunk
            let mut raw_nbt = Vec::new();
            write_named_tag(&mut raw_nbt, name, tag)?;

            let mut compressed = Vec::new();
            let mut encoder = ZlibEncoder::new(&mut compressed, Compression::default());
            encoder.write_all(&raw_nbt)?;
            encoder.finish()?;

            let total_len = compressed.len() + 1; // +1 for compression type byte
            let sectors_needed = (total_len + 4 + SECTOR_SIZE - 1) / SECTOR_SIZE;

            locations[index] = ChunkLocation {
                offset: current_sector,
                sector_count: sectors_needed as u8,
            };

            // Write chunk data
            self.writer
                .seek(SeekFrom::Start(current_sector as u64 * SECTOR_SIZE as u64))?;
            self.writer.write_all(&(total_len as u32).to_be_bytes())?;
            self.writer.write_all(&[2u8])?; // Zlib
            self.writer.write_all(&compressed)?;

            // Pad to sector boundary
            let padding = (sectors_needed * SECTOR_SIZE) - (total_len + 4);
            if padding > 0 {
                self.writer.write_all(&vec![0u8; padding])?;
            }

            current_sector += sectors_needed as u32;
        }

        // Write headers back at start
        self.writer.seek(SeekFrom::Start(0))?;
        for loc in &locations {
            let mut buf = [0u8; 4];
            buf[0] = ((loc.offset >> 16) & 0xFF) as u8;
            buf[1] = ((loc.offset >> 8) & 0xFF) as u8;
            buf[2] = (loc.offset & 0xFF) as u8;
            buf[3] = loc.sector_count;
            self.writer.write_all(&buf)?;
        }

        // Timestamps (just use 0 for now)
        for _ in 0..1024 {
            self.writer.write_all(&[0u8; 4])?;
        }

        Ok(())
    }
}
