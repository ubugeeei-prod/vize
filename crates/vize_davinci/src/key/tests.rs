use core::fmt::Write as _;

use vize_s0::{Span, String};

use super::{ArtifactKey, KeySink, KeyedArtifact, rebase, schema, source_block_key};
use crate::stage::Stage;

/// A page of typed fields: a text and one span.
struct Page {
    text: &'static str,
    span: Span,
}

impl KeyedArtifact for Page {
    const STAGE: Stage = Stage::Semantic;
    const SCHEMA_VERSION: u32 = 7;

    fn feed_key(&self, sink: &mut KeySink) {
        sink.feed_str(self.text);
        sink.feed_span(self.span);
    }
}

fn display(key: ArtifactKey) -> String {
    let mut out = String::default();
    write!(out, "{key}").expect("string write");
    out
}

#[test]
fn keys_print_stage_version_and_the_full_digest() {
    let key = ArtifactKey::of(
        &Page {
            text: "a",
            span: Span::new(10, 12),
        },
        10,
    );
    let mut expected = String::from("s2.v7:");
    for byte in key.hash() {
        write!(expected, "{byte:02x}").expect("string write");
    }
    assert_eq!(display(key), expected);
    assert_eq!((key.stage(), key.schema_version()), (Stage::Semantic, 7));
}

#[test]
fn spans_key_relative_to_their_block_start() {
    let at = |start: u32| {
        ArtifactKey::of(
            &Page {
                text: "a",
                span: Span::new(start + 3, start + 5),
            },
            start,
        )
    };
    assert_eq!(at(0), at(1000));
    let moved_inside = ArtifactKey::of(
        &Page {
            text: "a",
            span: Span::new(4, 6),
        },
        0,
    );
    assert_ne!(at(0), moved_inside);
}

#[test]
fn out_of_block_spans_do_not_collapse_onto_relative_ones() {
    assert_eq!(rebase(Span::new(3, 5), 4), None);
    assert_eq!(rebase(Span::new(4, 3), 4), None);
    assert_eq!(rebase(Span::new(6, 9), 4), Some(Span::new(2, 5)));
    let key = |span: Span| ArtifactKey::of(&Page { text: "a", span }, 4);
    // Saturating would map both of these onto `0..1`.
    assert_ne!(key(Span::new(2, 5)), key(Span::new(4, 5)));
}

#[test]
fn stage_and_version_are_inside_the_hashed_domain() {
    let digest = |stage: Stage, version: u32| {
        let mut sink = KeySink::new(stage, version, 0);
        sink.feed_str("same page");
        sink.finish().hash()
    };
    let base = digest(Stage::Surface, 1);
    assert_ne!(base, digest(Stage::Semantic, 1));
    assert_ne!(base, digest(Stage::Surface, 2));
}

#[test]
fn length_prefixes_keep_adjacent_fields_apart() {
    let fields = |a: &str, b: &str| {
        let mut sink = KeySink::new(Stage::Source, 1, 0);
        sink.feed_str(a);
        sink.feed_str(b);
        sink.finish()
    };
    assert_ne!(fields("ab", "c"), fields("a", "bc"));
}

#[test]
fn source_block_headers_are_an_attribute_set() {
    let content = "const a = 1\n";
    let setup_ts = [("setup", Some("")), ("lang", Some("ts"))];
    let ts_setup = [("lang", Some("ts")), ("setup", Some(""))];
    let key = source_block_key("script", &setup_ts, content);
    assert_eq!(key, source_block_key("script", &ts_setup, content));
    assert_eq!(
        (key.stage(), key.schema_version()),
        (Stage::Source, schema::SOURCE_BLOCK)
    );
    assert_ne!(
        key,
        source_block_key("script", &[("lang", Some("ts"))], content)
    );
    assert_ne!(
        source_block_key("style", &[("scoped", None)], ""),
        source_block_key("style", &[("scoped", Some(""))], "")
    );
    assert_ne!(key, source_block_key("style", &setup_ts, content));
}
