use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use indexmap::IndexMap;
use mcse_nbt::nbt::NbtTag;
use mcse_nbt::nbt::encode::write_named_tag;
use mcse_nbt::nbt::parse::parse_named_tag;
use nom::error::Error;
use std::io::{Cursor, Read, Write};

#[test]
fn test_complex_nbt_round_trip_gzip() {
    let mut root_map = IndexMap::new();

    // Some basic types
    root_map.insert("byte".to_string(), NbtTag::Byte(127));
    root_map.insert("short".to_string(), NbtTag::Short(32767));
    root_map.insert("int".to_string(), NbtTag::Int(2147483647));

    // List of Strings
    let list = vec![
        NbtTag::String("A".to_string()),
        NbtTag::String("B".to_string()),
        NbtTag::String("C".to_string()),
    ];
    root_map.insert("list".to_string(), NbtTag::List(list));

    // Arrays
    root_map.insert("intArray".to_string(), NbtTag::IntArray(vec![1, 2, 3]));

    // Compound
    let mut nested = IndexMap::new();
    nested.insert("key".to_string(), NbtTag::String("value".to_string()));
    root_map.insert("nested".to_string(), NbtTag::Compound(nested));

    let root = NbtTag::Compound(root_map);

    // 1. Encode
    let mut raw_buf = Vec::new();
    write_named_tag(&mut raw_buf, "Level", &root).expect("Failed to encode");

    // 2. Gzip (Simulating level.dat)
    let mut gzipped = Vec::new();
    let mut encoder = GzEncoder::new(&mut gzipped, Compression::default());
    encoder.write_all(&raw_buf).expect("Failed to gzip");
    encoder.finish().expect("Failed to finish gzip");

    // 3. Gunzip
    let mut decoder = GzDecoder::new(&gzipped[..]);
    let mut unzipped = Vec::new();
    decoder
        .read_to_end(&mut unzipped)
        .expect("Failed to gunzip");
    assert_eq!(unzipped, raw_buf);

    // 4. Decode
    let mut input = &unzipped[..];
    let (name, decoded) = parse_named_tag::<Error<&[u8]>>(&mut input).expect("Failed to decode");

    assert_eq!(name, "Level");
    assert_eq!(decoded, root);
}

#[test]
fn test_anvil_round_trip() {
    use mcse_nbt::anvil::access::Region;
    use mcse_nbt::anvil::encode::RegionWriter;

    let temp_dir = std::env::temp_dir();
    let mca_path = temp_dir.join("test.mca");

    let mut chunks = Vec::new();
    let mut map = IndexMap::new();
    map.insert("Data".to_string(), NbtTag::Int(123));
    chunks.push((0, 0, "Chunk".to_string(), NbtTag::Compound(map)));

    // 1. Write
    {
        let file = std::fs::File::create(&mca_path).unwrap();
        let mut writer = RegionWriter::new(file);
        writer.write_all_chunks(&chunks).unwrap();
    }

    // 2. Read
    {
        let region = Region::open(&mca_path).unwrap();
        let (name, tag) = region.get_chunk_nbt(0, 0).unwrap().unwrap();
        assert_eq!(name, "Chunk");
        if let NbtTag::Compound(m) = tag {
            assert_eq!(m.get("Data"), Some(&NbtTag::Int(123)));
        } else {
            panic!("Not a compound");
        }
    }

    std::fs::remove_file(mca_path).ok();
}
