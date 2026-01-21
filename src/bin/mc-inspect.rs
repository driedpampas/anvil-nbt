// Copyright 2026 driedpampas@proton.me
// SPDX-License-Identifier: GPL-3.0-or-later

use anvil_nbt::anvil::access::Region;
use anvil_nbt::nbt::parse::parse_named_tag;
use clap::{Parser, Subcommand};
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{Read, Write};
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

fn main() {
    if let Err(e) = run() {
        let msg = format!("{:?}", e).to_lowercase();
        if msg.contains("broken pipe") || msg.contains("os error 32") {
            std::process::exit(0);
        }
        eprintln!("Error: {:?}", e);
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();

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
            let (name, tag) =
                parse_named_tag(&mut input).map_err(|_| anyhow::anyhow!("Failed to parse NBT"))?;
            writeln!(handle, "Root tag name: '{}'", name)?;
            writeln!(handle, "{:#?}", tag)?;
        }
        Commands::Anvil { path, x, z } => {
            let region = Region::open(path)?;
            if let (Some(x), Some(z)) = (x, z) {
                if let Some((name, tag)) = region.get_chunk_nbt(x, z)? {
                    writeln!(handle, "Chunk ({}, {}) root tag name: '{}'", x, z, name)?;
                    writeln!(handle, "{:#?}", tag)?;
                } else {
                    writeln!(
                        handle,
                        "Chunk ({}, {}) is not present in this region.",
                        x, z
                    )?;
                }
            } else {
                writeln!(
                    handle,
                    "Anvil region file loaded. Use -x and -z to inspect a specific chunk."
                )?;
            }
        }
    }
    Ok(())
}
