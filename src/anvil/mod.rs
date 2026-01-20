pub mod access;
pub mod encode;

pub const SECTOR_SIZE: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChunkLocation {
    pub offset: u32,      // In sectors
    pub sector_count: u8, // In sectors
}

#[derive(Debug, Clone)]
pub struct RegionHeader {
    pub locations: [ChunkLocation; 1024],
    pub timestamps: [u32; 1024],
}

pub enum CompressionType {
    Gzip = 1,
    Zlib = 2,
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
