//! Intentional P4-13 divergences from the retired hand scanner, each pinned
//! exactly. D1–D4 are the eight corpus fixtures slice 1's differential lane
//! ledgered (the P4-13 record lists them); the rest are scanner bugs no
//! committed fixture happened to exercise. Every case is a place where the
//! substring scanner read something other than what the SFC/HTML syntax
//! says — the S0/S1 reading is the authored structure.

use super::parse_art;
use crate::types::{ArtDescriptor, ArtParseError, ArtParseOptions};
use vize_s0::Allocator;

fn parse<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> Result<ArtDescriptor<'a>, ArtParseError> {
    parse_art(allocator, source, ArtParseOptions::default())
}

fn variant_shape<'a>(desc: &ArtDescriptor<'a>) -> std::vec::Vec<(&'a str, &'a str)> {
    desc.variants.iter().map(|v| (v.name, v.template)).collect()
}

/// D1 — `<variant … />` is an empty variant (HTML self-closing syntax, as
/// the Patina Musea rules already read it); the scanner hunted for a
/// `</variant>` that does not exist and failed the file.
#[test]
fn d1_self_closing_variants_are_empty_variants() {
    let allocator = Allocator::new();
    let source = include_str!(
        "../../tests/fixtures/differential/content-musea-and-css-1e05f9370f2a.art.vue.txt"
    );
    let desc = parse(&allocator, source).unwrap();
    assert_eq!(variant_shape(&desc), [("primary", ""), ("secondary", "")]);
    let locs: std::vec::Vec<_> = desc
        .variants
        .iter()
        .map(|v| v.loc.map(|loc| (loc.start, loc.end, loc.start_line)))
        .collect();
    assert_eq!(locs, [Some((48, 74, 2)), Some((77, 105, 3))]);
}

/// D2 — `title = "Button"` is the `title` attribute (whitespace around
/// `=` is HTML syntax); the scanner required `title=` and reported
/// `MissingTitle`.
#[test]
fn d2_spaced_equals_is_an_attribute_value() {
    let allocator = Allocator::new();
    let source = include_str!(
        "../../tests/fixtures/differential/vize-patina-require-title-45a52dcf3b55.art.vue.txt"
    );
    let desc = parse(&allocator, source).unwrap();
    assert_eq!(
        (desc.metadata.title, desc.metadata.component),
        ("Button", Some("./Button.vue"))
    );
    assert_eq!(desc.variants.len(), 0);
}

/// D3 — a `<style>` inside `<art>` is art-block content, not an SFC style
/// block; the scanner collected every `<style` substring anywhere.
#[test]
fn d3_style_inside_art_is_not_an_sfc_style_block() {
    let allocator = Allocator::new();
    let source = include_str!(
        "../../tests/fixtures/differential/vize-maestro-configured-lint-tests-8f44edc047c4.art.vue.txt"
    );
    let desc = parse(&allocator, source).unwrap();
    assert_eq!(desc.metadata.title, "Inline");
    assert_eq!(desc.styles.len(), 0);
    assert_eq!(desc.variants.len(), 0);
}

/// D4 — input the SFC splitter cannot split reports the splitter's error;
/// the scanner answered `NoArtBlock` because `<article` is not `<art `.
#[test]
fn d4_unsplittable_container_reports_the_container_error() {
    let allocator = Allocator::new();
    let source =
        include_str!("../../tests/fixtures/differential/vize-patina-html-66050b9db3b6.art.vue.txt");
    let Err(ArtParseError::ParseError { line, message }) = parse(&allocator, source) else {
        panic!("expected the container error");
    };
    assert_eq!(
        (line, message.as_str()),
        (
            1,
            "Malformed <article> block: the opening tag is incomplete."
        )
    );
}

/// An unterminated `<art>` stays `NoArtBlock` (the long-standing contract
/// the Vite plugin reports), not a container error.
#[test]
fn unterminated_art_is_still_no_art_block() {
    let allocator = Allocator::new();
    for source in [
        "<art>",
        "<art title=\"A\">\n<variant name=\"v\"></variant>",
        "<art",
    ] {
        assert!(
            matches!(parse(&allocator, source), Err(ArtParseError::NoArtBlock)),
            "{source:?}"
        );
    }
}

/// A commented-out variant is a comment, not a variant.
#[test]
fn commented_out_variants_are_ignored() {
    let allocator = Allocator::new();
    let source = "<art title=\"A\">\n<!-- <variant name=\"Old\">o</variant> -->\n<variant name=\"New\">n</variant>\n</art>";
    let desc = parse(&allocator, source).unwrap();
    assert_eq!(variant_shape(&desc), [("New", "n")]);
}

/// `<article>` before the `<art>` block (an inline-art `.vue` template) no
/// longer hides the block; `<art` inside script text is not a block.
#[test]
fn art_block_is_found_by_block_structure() {
    let allocator = Allocator::new();
    let source = "<script setup>\nconst s = \"<art title='x'></art>\";\n</script>\n<template><article>t</article></template>\n<art title=\"Inline\">\n<variant name=\"v\">x</variant>\n</art>";
    let desc = parse(&allocator, source).unwrap();
    assert_eq!(desc.metadata.title, "Inline");
    assert_eq!(variant_shape(&desc), [("v", "x")]);
    assert_eq!(
        desc.script_setup.map(|script| script.content),
        Some("const s = \"<art title='x'></art>\";")
    );
}

/// Attribute values are read by HTML attribute syntax: a quoted value may
/// hold `>` and `name=`-shaped text, an unquoted value keeps its `/`, and
/// a boolean name inside another value is not that attribute.
#[test]
fn attribute_values_follow_html_syntax() {
    let allocator = Allocator::new();
    let source = "<art description=\"a > b, not title=x\" title=T>\n<variant name=a/b args='{\"k\": \"the default\"}'>x</variant>\n</art>";
    let desc = parse(&allocator, source).unwrap();
    assert_eq!(
        (desc.metadata.title, desc.metadata.description),
        ("T", Some("a > b, not title=x"))
    );
    assert_eq!(variant_shape(&desc), [("a/b", "x")]);
    assert!(!desc.variants[0].is_default);
    assert_eq!(desc.variants[0].args.len(), 1);
    assert_eq!(
        desc.variants[0].args.get("k"),
        Some(&serde_json::json!("the default"))
    );
}

/// Inherited S0 limitation (recorded in the P4-13 record): the shared SFC
/// splitter ends an unquoted block-tag value at `/`, so an `<art>` open tag
/// with `component=./Button.vue` is a malformed block — `NoArtBlock`. The
/// scanner read the value as `"."`. Quoted values (every fixture) are
/// unaffected; the splitter fix is a compiler-parity change of its own.
#[test]
fn unquoted_slash_in_the_art_open_tag_is_a_splitter_limitation() {
    let allocator = Allocator::new();
    let source = "<art title=T component=./Button.vue>\n<variant name=\"v\">x</variant>\n</art>";
    assert!(matches!(
        parse(&allocator, source),
        Err(ArtParseError::NoArtBlock)
    ));
}

/// CRLF sources: `<art` / `<variant` followed by a line break are tags.
#[test]
fn crlf_after_tag_names_is_whitespace() {
    let allocator = Allocator::new();
    let source = "<art\r\n  title=\"A\">\r\n<variant\r\n  name=\"v\">x</variant >\r\n</art>\r\n";
    let desc = parse(&allocator, source).unwrap();
    assert_eq!(desc.metadata.title, "A");
    assert_eq!(variant_shape(&desc), [("v", "x")]);
}
