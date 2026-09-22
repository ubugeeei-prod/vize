//! P5-6a: the SFC descriptor query — one parse per buffer revision, exact
//! accounting, and every served descriptor equal to the clean parse.

use core::fmt::Write as _;

use vize_resident::descriptor::parse_descriptor;
use vize_resident::{DescriptorStats, ParsedSfc, ResidentDocuments, SharedDescriptor};
use vize_s0::String;

const SFC: &str = "<script setup>\nconst n = 1\n</script>\n<template><p>{{ n }}</p></template>\n<style>p{}</style>\n";

fn stats(lookups: u32, parses: u32) -> DescriptorStats {
    DescriptorStats { lookups, parses }
}

/// The descriptor's full debug print: equal prints are equal descriptors.
fn print(descriptor: &SharedDescriptor) -> String {
    let mut out = String::default();
    write!(out, "{:?}", **descriptor).unwrap();
    out
}

fn edited(k: usize) -> String {
    let mut text = String::from(SFC);
    for _ in 0..=k % 2 {
        text.push(' ');
    }
    text
}

#[test]
fn lookups_between_two_edits_share_one_parse() {
    let mut docs = ResidentDocuments::default();
    for k in 0..4 {
        let text = edited(k);
        for _ in 0..3 {
            let _served = docs.descriptor("file:///a.vue", "/a.vue", &text);
        }
        assert_eq!(docs.take_stats(), stats(3, 1), "edit {k}");
    }
}

#[test]
fn every_served_descriptor_is_the_clean_parse() {
    let mut docs = ResidentDocuments::default();
    for k in 0..4 {
        let text = edited(k);
        let served = docs.descriptor("file:///a.vue", "/a.vue", &text).unwrap();
        let clean = parse_descriptor("/a.vue", &text).unwrap();
        assert_eq!(print(&served), print(&clean), "edit {k}");
        assert!(served == clean);
    }
}

#[test]
fn an_edit_to_one_document_leaves_the_others_unparsed() {
    let mut docs = ResidentDocuments::default();
    let _a = docs.descriptor("a", "/a.vue", SFC);
    let _b = docs.descriptor("b", "/b.vue", SFC);
    assert_eq!(docs.take_stats(), stats(2, 2));
    let _a = docs.descriptor("a", "/a.vue", &edited(0));
    let _b = docs.descriptor("b", "/b.vue", SFC);
    assert_eq!(
        docs.take_stats(),
        stats(2, 1),
        "b's memo is validated, not re-run"
    );
}

#[test]
fn the_filename_is_part_of_the_descriptor() {
    let mut docs = ResidentDocuments::default();
    let first = docs.descriptor("k", "/a.vue", SFC).unwrap();
    let renamed = docs.descriptor("k", "/b.vue", SFC).unwrap();
    assert_eq!(first.filename, "/a.vue");
    assert_eq!(renamed.filename, "/b.vue");
    assert!(first != renamed);
    assert_eq!(docs.take_stats(), stats(2, 2));
}

#[test]
fn a_rejected_buffer_is_parsed_once_and_keeps_the_error() {
    let mut docs = ResidentDocuments::default();
    let text = "<template><div></div>";
    let first = docs.parsed("file:///Broken.vue", "/Broken.vue", text);
    let again = docs.parsed("file:///Broken.vue", "/Broken.vue", text);
    assert_eq!(docs.take_stats(), stats(2, 1));
    let ParsedSfc::Failed(error) = &first else {
        panic!("unclosed template is a parse error, got {first:?}");
    };
    assert_eq!(
        error.message.as_str(),
        "Malformed <template> block: the closing tag is missing."
    );
    let loc = error.loc.expect("the parser reports a location");
    assert_eq!(
        (
            loc.start_line,
            loc.start_column,
            loc.end_line,
            loc.end_column
        ),
        (1, 1, 1, 22)
    );
    assert_eq!(again, first);
    assert!(
        docs.descriptor("file:///Broken.vue", "/Broken.vue", text)
            .is_none()
    );
    assert_eq!(docs.take_stats(), stats(1, 0), "the rejection is the memo");

    let fixed = "<template><div></div></template>\n";
    let repaired = docs.parsed("file:///Broken.vue", "/Broken.vue", fixed);
    assert!(matches!(repaired, ParsedSfc::Descriptor(_)));
    assert_eq!(docs.take_stats(), stats(1, 1));
}

#[test]
fn closing_releases_the_buffer_and_reopening_parses_again() {
    let mut docs = ResidentDocuments::default();
    let _open = docs.descriptor("a", "/a.vue", SFC);
    docs.close("a");
    let _closed = docs.take_stats();
    let reopened = docs.descriptor("a", "/a.vue", SFC).unwrap();
    assert_eq!(docs.take_stats(), stats(1, 1));
    assert_eq!(
        print(&reopened),
        print(&parse_descriptor("/a.vue", SFC).unwrap())
    );
}
