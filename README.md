# anvil-nbt

[![Docs](https://docs.rs/anvil-nbt/badge.svg)](https://docs.rs/anvil-nbt)
[![License](https://img.shields.io/badge/license-GPL--3.0--or--later-blue.svg)](LICENSE)

A Rust library for parsing and encoding Minecraft's **NBT** and **Anvil (.mca)** formats.

Built for world editors, servers, and tools that need reliable access to Minecraft world data.

## Features

- **High Performance**: Built on `nom` with trait-based parsers for efficient parsing
- **Lazy Loading**: Memory-mapped Anvil region files via `memmap2` load only the chunks you need
- **Full NBT Support**: Handles all tag types, including Modified UTF-8 (MUTF-8) strings
- **Bit-Perfect Round-trips**: Idempotent parsers and encoders preserve data exactly
- **Compression Support**: Built-in Gzip and Zlib compression handling via `flate2`
- **CLI Utility**: Includes `mc-inspect` for inspecting world files from the terminal

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
anvil-nbt = "0.1.0"
```

## Quick Start

### Reading a `level.dat` (Gzipped NBT)

```rust
use anvil_nbt::nbt::parse::parse_named_tag;
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::Read;

fn main() -> anyhow::Result<()> {
    let file = File::open("level.dat")?;
    let mut decoder = GzDecoder::new(file);
    let mut data = Vec::new();
    decoder.read_to_end(&mut data)?;

    let mut input = &data[..];
    let (name, tag) = parse_named_tag::<nom::error::Error<&[u8]>>(&mut input)
        .map_err(|e| anyhow::anyhow!("Parse error: {:?}", e))?;

    println!("Root tag name: {}", name);
    println!("{:#?}", tag);
    Ok(())
}
```

### Accessing an Anvil Region File

```rust
use anvil_nbt::anvil::access::Region;

fn main() -> anyhow::Result<()> {
    let region = Region::open("r.0.0.mca")?;
    
    // Get chunk at (5, 10) within this region
    if let Some((name, tag)) = region.get_chunk_nbt(5, 10)? {
        println!("Chunk (5,10) root: {}", name);
        // Do something with the NBT data!
    }
    
    Ok(())
}
```

## CLI Utility: mc-inspect

Inspect Minecraft files directly from your terminal:

```bash
# Install the CLI tool
cargo install --path .

# Inspect a level.dat file
mc-inspect nbt level.dat

# Peek at a specific chunk in an Anvil file
mc-inspect anvil r.0.0.mca -x 5 -z 10
```

## License

This project is licensed under the GPL-3.0-or-later License - see the [LICENSE](LICENSE) file for details.
