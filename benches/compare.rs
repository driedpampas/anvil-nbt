// Copyright 2026 driedpampas@proton.me
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    fs::File,
    hint::black_box,
    io::{Cursor, Read},
};

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use flate2::read::GzDecoder;

fn bench_read_file(filename: &str, c: &mut Criterion) {
    // Try .local/ first, then tests/
    let path = if std::path::Path::new(&format!(".local/{filename}")).exists() {
        format!(".local/{filename}")
    } else {
        format!("tests/{filename}")
    };

    let mut file = File::open(&path).unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();
    let mut src = &contents[..];

    // decode the original src so most of the time isn't spent on unzipping
    let mut src_decoder = GzDecoder::new(&mut src);
    let mut input = Vec::new();
    if src_decoder.read_to_end(&mut input).is_err() {
        // oh probably wasn't gzipped then
        input = contents;
    }

    let mut input_stream = Cursor::new(&input[..]);

    let mut group = c.benchmark_group(format!("compare/{filename}"));
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("anvil_parse", |b| {
        b.iter(|| {
            let mut input = &input[..];
            black_box(
                anvil_nbt::nbt::parse::parse_named_tag::<nom::error::Error<&[u8]>>(&mut input)
                    .unwrap(),
            );
        })
    });

    /*
    group.bench_function("simdnbt_borrow_parse", |b| {
        b.iter(|| {
            black_box(simdnbt::borrow::read(&mut input_stream).unwrap());
            input_stream.set_position(0);
        })
    });
    group.bench_function("simdnbt_owned_parse", |b| {
        b.iter(|| {
            black_box(simdnbt::owned::read(&mut input_stream).unwrap());
            input_stream.set_position(0);
        })
    });
    */
    group.bench_function("shen_parse", |b| {
        let mut input = input.clone();
        b.iter(|| {
            let nbt = shen_nbt5::NbtValue::from_binary::<shen_nbt5::nbt_version::Java>(&mut input)
                .unwrap();
            black_box(nbt);
        })
    });
    /*
    group.bench_function("azalea_parse", |b| {
        b.iter(|| {
            black_box(azalea_nbt::Nbt::read(&mut input_stream).unwrap());
            input_stream.set_position(0);
        })
    });
    */
    group.bench_function("graphite_parse", |b| {
        b.iter(|| {
            black_box(graphite_binary::nbt::decode::read(&mut &input[..]).unwrap());
        })
    });
    group.bench_function("valence_parse", |b| {
        b.iter(|| {
            let nbt = valence_nbt::from_binary::<String>(&mut &input[..]).unwrap();
            black_box(nbt);
        })
    });
    group.bench_function("fastnbt_parse", |b| {
        b.iter(|| {
            let nbt: fastnbt::Value = fastnbt::from_bytes(&input).unwrap();
            black_box(nbt);
        })
    });
    group.bench_function("hematite_parse", |b| {
        b.iter(|| {
            black_box(nbt::Blob::from_reader(&mut input_stream).unwrap());
            input_stream.set_position(0);
        })
    });
    group.bench_function("crab_parse", |b| {
        b.iter(|| {
            black_box(crab_nbt::Nbt::read(&mut input_stream).unwrap());
            input_stream.set_position(0);
        })
    });
    group.bench_function("ussr_borrow_parse", |b| {
        b.iter(|| {
            black_box(ussr_nbt::borrow::Nbt::read(&mut input_stream).unwrap());
            input_stream.set_position(0);
        })
    });

    /*
    let nbt_azalea = azalea_nbt::Nbt::read(&mut Cursor::new(&input)).unwrap();
    group.bench_function("azalea_write", |b| {
        b.iter(|| {
            let mut out = Vec::new();
            nbt_azalea.write(&mut out);
            black_box(out);
        })
    });
    */

    /*
    let nbt_simdnbt_borrow = simdnbt::borrow::read(&mut Cursor::new(&input))
        .unwrap()
        .unwrap()
        .unwrap();
    group.bench_function("simdnbt_borrow_write", |b| {
        b.iter(|| {
            let mut out = Vec::new();
            nbt_simdnbt_borrow.write(&mut out);
            black_box(out);
        })
    });

    let nbt_simdnbt_owned = simdnbt::owned::read(&mut Cursor::new(&input))
        .unwrap()
        .unwrap()
        .unwrap();
    group.bench_function("simdnbt_owned_write", |b| {
        b.iter(|| {
            let mut out = Vec::new();
            nbt_simdnbt_owned.write(&mut out);
            black_box(out);
        })
    });
    */

    let nbt_graphite = graphite_binary::nbt::decode::read(&mut &input[..]).unwrap();
    group.bench_function("graphite_write", |b| {
        b.iter(|| {
            let out = graphite_binary::nbt::encode::write(&nbt_graphite);
            black_box(out);
        })
    });

    let (name, nbt_anvil) =
        anvil_nbt::nbt::parse::parse_named_tag::<nom::error::Error<&[u8]>>(&mut &input[..])
            .unwrap();
    group.bench_function("anvil_write", |b| {
        b.iter(|| {
            let mut out = Vec::new();
            anvil_nbt::nbt::encode::write_named_tag(&mut out, &name, &nbt_anvil).unwrap();
            black_box(out);
        })
    });
}

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn bench(c: &mut Criterion) {
    bench_read_file("complex_player.dat", c);
}

criterion_group!(compare, bench);
criterion_main!(compare);
