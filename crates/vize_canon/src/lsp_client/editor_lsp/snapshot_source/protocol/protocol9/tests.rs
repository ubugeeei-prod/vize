//! Actual pinned beta binary fixture, independent source and malformed envelopes.
use super::super::{SourceTextRefusal, source_text};
use serde_json::Value;
use sha2::{Digest, Sha256};

const BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/snapshot-source-text/protocol9-beta.bin"
));
fn fixture() -> Value {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/typechecker/snapshot-source-text/protocol9-beta.json"
    )))
    .unwrap()
}
fn word(offset: usize) -> usize {
    u32::from_le_bytes(BYTES.get(offset..offset + 4).unwrap().try_into().unwrap()) as usize
}
fn mutate(offset: usize, value: u32) -> Vec<u8> {
    let mut bytes = BYTES.to_vec();
    bytes
        .get_mut(offset..offset + 4)
        .unwrap()
        .copy_from_slice(&value.to_le_bytes());
    bytes
}
#[test]
fn actual_beta_complete_text_is_independent_of_the_opaque_ast_and_source_id() {
    let f = fixture();
    let name = f["file"].as_str().unwrap();
    let expected = "/*🧭é日本語*/\r\nexport const label: string = \"ok\";\r\nexport const length: number = label;\u{2028}export const x = 1;\u{2029}";
    assert_eq!(f["text"], expected);
    assert_eq!(expected.encode_utf16().count(), 105);
    assert_ne!(expected.len(), 105);
    let sha = Sha256::digest(BYTES)
        .iter()
        .map(|byte| vize_l0::cstr!("{byte:02x}"))
        .collect::<vize_l0::String>();
    assert_eq!(sha.as_str(), f["binarySha256"].as_str().unwrap());
    assert_eq!(source_text(BYTES, name), Ok((expected, name, name)));
    // A runtime-owned opaque source ID is not a text identity or coordinate.
    assert_eq!(
        source_text(&mutate(44, u32::MAX), name),
        Ok((expected, name, name))
    );
}
#[test]
fn unknown_versions_reserved_bits_truncation_and_bad_section_arithmetic_refuse() {
    let f = fixture();
    let name = f["file"].as_str().unwrap();
    for version in [0, 5, 6, 7, 8, 10, 9 << 24 | 1] {
        assert!(source_text(&mutate(0, version), name).is_err());
    }
    for end in 0..64 {
        assert_eq!(
            source_text(BYTES.get(..end).unwrap(), name),
            Err(SourceTextRefusal::Header)
        );
    }
    for end in [word(40), word(40) + 28, word(40) + 55, BYTES.len() - 1] {
        assert!(source_text(BYTES.get(..end).unwrap(), name).is_err());
    }
    for offset in [24, 28, 32, 36, 40, 60] {
        assert!(source_text(&mutate(offset, u32::MAX), name).is_err());
    }
    for (offset, value) in [
        (20, 4),
        (24, 44),
        (28, 63),
        (32, 0),
        (36, word(32) as u32 + 75),
        (60, word(40) as u32 + 1),
    ] {
        assert!(source_text(&mutate(offset, value), name).is_err());
    }
    let mut longer = BYTES.to_vec();
    longer.push(0);
    assert!(source_text(&longer, name).is_err());
}
#[test]
fn root_source_span_kind_sentinel_and_all_known_references_are_bounded() {
    let f = fixture();
    let name = f["file"].as_str().unwrap();
    let nodes = word(40);
    let root = nodes + 28;
    let extended = word(32);
    for (offset, value) in [
        (nodes, 1),
        (root, 307),
        (root + 4, 1),
        (root + 8, 104),
        (root + 12, 1),
        (root + 16, 1),
        (root + 20, 0x80000004),
        (extended + 44, u32::MAX),
    ] {
        assert!(source_text(&mutate(offset, value), name).is_err());
    }
    for field in [20, 24, 28, 32, 36, 40, 52, 56, 72] {
        assert!(source_text(&mutate(extended + field, u32::MAX - 1), name).is_err());
    }
    for field in [0, 4, 8, 48, 60, 64, 68] {
        assert!(source_text(&mutate(extended + field, 1), name).is_err());
        assert!(source_text(&mutate(extended + field, u32::MAX - 1), name).is_err());
    }
}
#[test]
fn utf8_line_endings_neighbor_pairs_and_exact_source_identity_are_not_repaired() {
    let f = fixture();
    let name = f["file"].as_str().unwrap();
    let strings = word(28);
    let table = word(24);
    let mut invalid = BYTES.to_vec();
    *invalid.get_mut(strings).unwrap() = 0xff;
    assert_eq!(source_text(&invalid, name), Err(SourceTextRefusal::Utf8));
    assert_eq!(
        source_text(BYTES, "/foreign/main.ts"),
        Err(SourceTextRefusal::SourceIdentity)
    );
    for offset in (table..strings).step_by(8) {
        assert!(source_text(&mutate(offset, u32::MAX), name).is_err());
        assert!(source_text(&mutate(offset + 4, u32::MAX), name).is_err());
    }
    let mut invalid = BYTES.to_vec();
    let text_end = strings + word(table + 4);
    // Replacing a three-byte LS with an invalid continuation sequence cannot
    // turn an ECMA line break into an accepted shifted source.
    *invalid.get_mut(text_end - 1).unwrap() = 0xff;
    assert_eq!(source_text(&invalid, name), Err(SourceTextRefusal::Utf8));
}
