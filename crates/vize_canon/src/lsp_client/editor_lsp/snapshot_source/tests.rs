//! Independently authored protocol envelope and full malformed/identity controls.

use super::{SourceTextRefusal, protocol::source_text, source_members};
use serde_json::Value;
use sha2::{Digest, Sha256};
use vize_l0::String;

const PAYLOAD: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/snapshot-source-text/protocol5-text.bin"
));
const NAME: &str = "/workspace/a #.ts";

#[test]
fn independently_authored_unicode_crlf_text_and_identities_are_byte_exact() {
    let fixture: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/typechecker/snapshot-source-text/protocol5-text.json"
    )))
    .unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(PAYLOAD)),
        fixture["sha256"].as_str().unwrap()
    );
    assert_eq!(
        source_text(PAYLOAD, NAME),
        Ok((
            fixture["text"].as_str().unwrap(),
            fixture["fileName"].as_str().unwrap(),
            fixture["path"].as_str().unwrap()
        ))
    );
}

#[test]
fn unsupported_metadata_and_parse_options_refuse_the_whole_view() {
    for (offset, value, refusal) in [
        (0, 4 << 24, SourceTextRefusal::Version),
        (0, 5, SourceTextRefusal::Version),
        (0, (5 << 24) | 1, SourceTextRefusal::Version),
        (20, 4, SourceTextRefusal::Header),
    ] {
        assert_refused(offset, value, refusal);
    }
    for end in 0..44 {
        assert_eq!(
            source_text(&PAYLOAD[..end], NAME),
            Err(SourceTextRefusal::Header)
        );
    }
}

#[test]
fn sections_cannot_escape_overlap_or_truncate_the_selected_root() {
    for offset in [24, 28, 32, 36, 40] {
        assert_refused(offset, u32::MAX, SourceTextRefusal::Sections);
    }
    assert_refused(24, 40, SourceTextRefusal::Sections);
    assert_refused(28, 69, SourceTextRefusal::Sections);
    assert_refused(36, word(32) + 47, SourceTextRefusal::Sections);
    let nodes = word(40) as usize;
    assert_eq!(
        source_text(&PAYLOAD[..nodes], NAME),
        Err(SourceTextRefusal::Sections)
    );
    assert_eq!(
        source_text(&PAYLOAD[..PAYLOAD.len() - 1], NAME),
        Err(SourceTextRefusal::Sections)
    );
    let mut extended = PAYLOAD.to_vec();
    extended.push(0);
    assert_eq!(
        source_text(&extended, NAME),
        Err(SourceTextRefusal::Sections)
    );
}

#[test]
fn source_root_kind_sentinel_span_and_extended_references_are_checked() {
    let nodes = word(40) as usize;
    let root = nodes + 28;
    let extended = word(32) as usize;
    for (offset, value) in [
        (nodes, 1),
        (root, 1),
        (root + 4, 1),
        (root + 8, 1),
        (root + 12, 1),
        (root + 16, 1),
        (root + 20, 0x40000000),
        (root + 20, 0x81000000),
        (root + 20, 0x80000004),
        (extended + 44, 2),
    ] {
        assert_refused(offset, value, SourceTextRefusal::Root);
    }
    for field in (20..=40).step_by(4) {
        assert_refused(extended + field, 0, SourceTextRefusal::Root);
    }
}

#[test]
fn every_string_pair_is_bounded_and_selected_indices_are_even() {
    let extended = word(32) as usize;
    for field in [0, 4, 8] {
        assert_refused(extended + field, 1, SourceTextRefusal::Strings);
        assert_refused(extended + field, u32::MAX - 1, SourceTextRefusal::Strings);
    }
    for pair in [44, 52, 60] {
        assert_refused(pair, u32::MAX, SourceTextRefusal::Strings);
        assert_refused(pair + 4, u32::MAX, SourceTextRefusal::Strings);
    }
}

#[test]
fn selected_utf8_is_strict_and_foreign_root_names_do_not_acquire_ownership() {
    let mut bytes = PAYLOAD.to_vec();
    let strings = word(28) as usize;
    bytes[strings] = 0xff;
    assert_eq!(source_text(&bytes, NAME), Err(SourceTextRefusal::Utf8));
    assert_eq!(
        source_text(PAYLOAD, "/foreign/a #.ts"),
        Err(SourceTextRefusal::SourceIdentity)
    );
    let file_name = strings + word(52) as usize;
    bytes = PAYLOAD.to_vec();
    bytes[file_name + 1] = b'X';
    assert_eq!(
        source_text(&bytes, NAME),
        Err(SourceTextRefusal::SourceIdentity)
    );
    let path = strings + word(60) as usize;
    bytes = PAYLOAD.to_vec();
    bytes[path + 1] = b'X';
    assert_eq!(
        source_text(&bytes, NAME),
        Err(SourceTextRefusal::SourceIdentity)
    );
}

#[test]
fn exact_native_uri_membership_refuses_dot_slash_percent_and_foreign_aliases() {
    let sources = source_members(&[String::from(NAME)]).unwrap();
    assert_eq!(
        sources.get("file:///workspace/a%20%23.ts"),
        Some(&String::from(NAME))
    );
    for uri in [
        "file:///workspace/./a%20%23.ts",
        "file:///workspace//a%20%23.ts",
        "file:///workspace/%61%20%23.ts",
        "file:///foreign/a%20%23.ts",
    ] {
        assert_eq!(sources.get(uri), None);
    }
    assert_eq!(
        source_members(&[String::from(NAME), String::from(NAME)]),
        None
    );
    assert_eq!(source_members(&[String::from("relative.ts")]), None);
    assert_eq!(source_members(&[String::from("/workspace/a\0.ts")]), None);
}

fn word(offset: usize) -> u32 {
    u32::from_le_bytes(PAYLOAD[offset..offset + 4].try_into().unwrap())
}

fn assert_refused(offset: usize, value: u32, expected: SourceTextRefusal) {
    let mut bytes = PAYLOAD.to_vec();
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    assert_eq!(
        source_text(&bytes, NAME),
        Err(expected),
        "offset {offset}, value {value}"
    );
}

#[cfg(unix)]
mod native;
