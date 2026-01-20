use clap::{Parser, Subcommand};
use flate2::read::GzDecoder;
use mcse_nbt::anvil::access::Region;
use mcse_nbt::nbt::parse::parse_named_tag;
use nom::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "mc-inspect")]
#[command(about = "Inspect Minecraft NBT and Anvil files", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inspect a .dat (NBT) file
    Nbt {
        /// Path to the .dat file
        path: PathBuf,
        /// Force uncompressed (if not gzipped)
        #[arg(short, long)]
        uncompressed: bool,
    },
    /// Inspect an .mca (Anvil) file
    Anvil {
        /// Path to the .mca file
        path: PathBuf,
        /// Chunk X coordinate
        #[arg(short, long)]
        x: Option<i32>,
        /// Chunk Z coordinate
        #[arg(short, long)]
        z: Option<i32>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Nbt { path, uncompressed } => {
            let mut file = File::open(path)?;
            let mut data = Vec::new();
            if uncompressed {
                file.read_to_end(&mut data)?;
            } else {
                let mut decoder = GzDecoder::new(file);
                decoder.read_to_end(&mut data)?;
            }

            let mut input = &data[..];
            let (name, tag) = parse_named_tag::<Error<&[u8]>>(&mut input)
                .map_err(|e| anyhow::anyhow!("Failed to parse NBT: {:?}", e))?;
            println!("Root tag name: '{}'", name);
            println!("{:#?}", tag);
        }
        Commands::Anvil { path, x, z } => {
            let region = Region::open(path)?;
            if let (Some(x), Some(z)) = (x, z) {
                if let Some((name, tag)) = region.get_chunk_nbt(x, z)? {
                    println!("Chunk ({}, {}) root tag name: '{}'", x, z, name);
                    println!("{:#?}", tag);
                } else {
                    println!("Chunk ({}, {}) is not present in this region.", x, z);
                }
            } else {
                println!("Anvil region file loaded. Use -x and -z to inspect a specific chunk.");
                // We could list non-empty chunks here
            }
        }
    }

    Ok(())
}
