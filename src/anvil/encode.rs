use crate::anvil::{ChunkLocation, SECTOR_SIZE};
use crate::nbt::NbtTag;
use crate::nbt::encode::write_named_tag;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::io::{Result, Seek, SeekFrom, Write};

pub struct RegionWriter<W: Write + Seek> {
    writer: W,
}

impl<W: Write + Seek> RegionWriter<W> {
    pub fn new(mut writer: W) -> Result<Self> {
        // Initial header space
        writer.write_all(&[0u8; SECTOR_SIZE * 2])?;
        Ok(RegionWriter { writer })
    }

    pub fn write_region(mut writer: W, chunks: &[(i32, i32, String, NbtTag)]) -> Result<()> {
        let mut locations = [ChunkLocation {
            offset: 0,
            sector_count: 0,
        }; 1024];
        let _timestamps = [0u32; 1024];

        writer.write_all(&[0u8; SECTOR_SIZE * 2])?;
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
            writer.seek(SeekFrom::Start(current_sector as u64 * SECTOR_SIZE as u64))?;
            writer.write_all(&(total_len as u32).to_be_bytes())?;
            writer.write_all(&[2u8])?; // Zlib
            writer.write_all(&compressed)?;

            // Pad to sector boundary
            let padding = (sectors_needed * SECTOR_SIZE) - (total_len + 4);
            if padding > 0 {
                writer.write_all(&vec![0u8; padding])?;
            }

            current_sector += sectors_needed as u32;
        }

        // Write headers back at start
        writer.seek(SeekFrom::Start(0))?;
        for loc in &locations {
            let mut buf = [0u8; 4];
            buf[0] = ((loc.offset >> 16) & 0xFF) as u8;
            buf[1] = ((loc.offset >> 8) & 0xFF) as u8;
            buf[2] = (loc.offset & 0xFF) as u8;
            buf[3] = loc.sector_count;
            writer.write_all(&buf)?;
        }

        // Timestamps (just use 0 or current time)
        for _ in 0..1024 {
            writer.write_all(&[0u8; 4])?;
        }

        Ok(())
    }
}
