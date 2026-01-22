// Copyright 2026 driedpampas@proton.me
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(feature = "serde")]
mod tests {
    use anvil_nbt::nbt::NbtTag;
    use anvil_nbt::nbt::serde_impl::{from_nbt, to_nbt};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestStruct {
        name: String,
        age: i32,
        active: bool,
        scores: Vec<i32>,
        metadata: Meta,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Meta {
        version: String,
        tags: Vec<String>,
    }

    #[test]
    fn test_struct_to_nbt_roundtrip() {
        let original = TestStruct {
            name: "Steve".to_owned(),
            age: 25,
            active: true,
            scores: vec![10, 20, 30],
            metadata: Meta {
                version: "1.0".to_owned(),
                tags: vec!["player".to_owned(), "admin".to_owned()],
            },
        };

        let nbt = to_nbt(&original).unwrap();
        let decoded: TestStruct = from_nbt(nbt).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_nbt_tag_json_roundtrip() {
        use indexmap::IndexMap;
        let mut map = IndexMap::new();
        map.insert("key".to_string(), NbtTag::String("value".to_string()));
        let original = NbtTag::Compound(map);

        let json = serde_json::to_string(&original).unwrap();
        let decoded: NbtTag = serde_json::from_str(&json).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_binary_roundtrip_via_serde() {
        use anvil_nbt::nbt::encode::write_named_tag;
        use anvil_nbt::nbt::parse::parse_named_tag;

        let original = TestStruct {
            name: "Alex".to_owned(),
            age: 30,
            active: false,
            scores: vec![1, 2, 3],
            metadata: Meta {
                version: "2.0".to_owned(),
                tags: vec!["vip".to_owned()],
            },
        };

        // Struct -> NbtTag
        let tag = to_nbt(&original).unwrap();

        // NbtTag -> Binary
        let mut buf = Vec::new();
        write_named_tag(&mut buf, "root", &tag).unwrap();

        // Binary -> NbtTag
        let mut input = &buf[..];
        let (name, decoded_tag) = parse_named_tag(&mut input).unwrap();
        assert_eq!(name, "root");

        // NbtTag -> Struct
        let decoded: TestStruct = from_nbt(decoded_tag).unwrap();

        assert_eq!(original, decoded);
    }
}
