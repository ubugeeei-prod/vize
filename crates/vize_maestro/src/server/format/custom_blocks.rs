//! Project supported Musea example formatting through the authored block span.

use tower_lsp::lsp_types::{Position, Range, TextEdit};
use vize_glyph::{EndOfLine, FormatOptions, FormatPreset, VueVersion};

use super::{on_type::format_on_type, range::format_range};

fn selected(source: &str, range: Range, options: &FormatOptions) -> Option<Vec<TextEdit>> {
    format_range(source, "/Examples.vue", range, options, VueVersion::V3)
}

fn apply(source: &str, edits: &[TextEdit]) -> Option<Vec<u8>> {
    let mut bytes = source.as_bytes().to_vec();
    for edit in edits.iter().rev() {
        let start = crate::ide::position_to_offset(
            source,
            edit.range.start.line,
            edit.range.start.character,
        )?;
        let end =
            crate::ide::position_to_offset(source, edit.range.end.line, edit.range.end.character)?;
        bytes.splice(start..end, edit.new_text.bytes());
    }
    Some(bytes)
}

#[test]
fn selected_art_uses_each_profile_without_editing_tags_or_neighbors() {
    const PREFIX: &str = "<template><p>🌸 untouched</p></template>\n<!--🌸--><art title=\"one\">";
    const BODY: &str = "<Button data-z=\"z\" data-a=\"a\" />";
    const SUFFIX: &str = "</art>\n<script setup>const   value=1</script>\n";
    let source = [PREFIX, BODY, SUFFIX].concat();
    // The comment's astral character occupies two UTF-16 units, not four bytes.
    let range = Range::new(Position::new(1, 27), Position::new(1, 28));
    let content = Range::new(
        Position::new(1, 26),
        Position::new(1, 26 + BODY.encode_utf16().count() as u32),
    );
    for (preset, expected) in [
        (
            FormatPreset::Vize,
            "\n  <Button data-a=\"a\" data-z=\"z\" />\n",
        ),
        (
            FormatPreset::Prettier,
            "\n  <Button data-z=\"z\" data-a=\"a\" />\n",
        ),
        (
            FormatPreset::Compact,
            "\n  <Button data-z=\"z\" data-a=\"a\" />\n",
        ),
        (
            FormatPreset::Documentation,
            "\n  <Button\n    data-z=\"z\"\n    data-a=\"a\"\n  />\n",
        ),
    ] {
        let options = FormatOptions::from_preset(preset);
        let edits = selected(&source, range, &options).unwrap();
        assert_eq!(
            edits,
            vec![TextEdit {
                range: content,
                new_text: expected.to_owned()
            }]
        );
        let bytes = apply(&source, &edits).unwrap();
        let applied = std::str::from_utf8(&bytes).unwrap();
        assert_eq!(applied, [PREFIX, expected, SUFFIX].concat());
        let caret = Range::new(Position::new(2, 2), Position::new(2, 3));
        for _ in 0..2 {
            assert_eq!(selected(applied, caret, &options), Some(Vec::new()));
        }
    }
}

#[test]
fn repeated_art_blocks_keep_their_identity_and_only_the_selected_one_changes() {
    const FIRST: &str = "<art title=\"first\"><Button   a=\"1\" /></art>\n";
    const SECOND: &str = "<art title=\"second\"><Other   b=\"2\" /></art>\n";
    let source = [FIRST, SECOND].concat();
    let range = Range::new(Position::new(1, 21), Position::new(1, 22));
    let options = FormatOptions::default();
    let edits = selected(&source, range, &options).unwrap();
    assert_eq!(
        edits,
        vec![TextEdit {
            range: Range::new(Position::new(1, 20), Position::new(1, 37)),
            new_text: "\n  <Other b=\"2\" />\n".to_owned(),
        }]
    );
    let bytes = apply(&source, &edits).unwrap();
    let applied = std::str::from_utf8(&bytes).unwrap();
    assert_eq!(
        applied,
        [
            FIRST,
            "<art title=\"second\">\n  <Other b=\"2\" />\n</art>\n"
        ]
        .concat()
    );
    assert_eq!(
        selected(
            applied,
            Range::new(Position::new(2, 2), Position::new(2, 3)),
            &options
        ),
        Some(Vec::new())
    );
}

#[test]
fn custom_art_ranges_preserve_crlf_and_reach_a_fixed_point() {
    let source = "<art>\r\n<Button   />\r\n</art>\r\n";
    let range = Range::new(Position::new(1, 1), Position::new(1, 2));
    let options = FormatOptions {
        end_of_line: EndOfLine::Auto,
        ..FormatOptions::default()
    };
    let edits = selected(source, range, &options).unwrap();
    assert_eq!(
        edits,
        vec![TextEdit {
            range: Range::new(Position::new(0, 5), Position::new(2, 0)),
            new_text: "\r\n  <Button />\r\n".to_owned(),
        }]
    );
    let bytes = apply(source, &edits).unwrap();
    let applied = std::str::from_utf8(&bytes).unwrap();
    assert_eq!(applied, "<art>\r\n  <Button />\r\n</art>\r\n");
    assert_eq!(selected(applied, range, &options), Some(Vec::new()));
}

#[test]
fn unknown_custom_blocks_remain_outside_the_supported_formatting_projection() {
    let source = "<docs>  exact raw 🌸  </docs>\n<art><Button   /></art>\n";
    let range = Range::new(Position::new(0, 7), Position::new(0, 8));
    assert_eq!(
        selected(source, range, &FormatOptions::default()),
        Some(Vec::new())
    );
}

#[test]
fn art_on_type_only_changes_indentation_and_preserves_in_progress_tokens() {
    let options = FormatOptions::default();
    let source = "<art>\n<Button />\n</art>\n";
    let position = Position::new(1, 2);
    let edits =
        format_on_type(source, "/Examples.vue", position, &options, VueVersion::V3).unwrap();
    assert_eq!(
        edits,
        vec![TextEdit {
            range: Range::new(Position::new(1, 0), Position::new(1, 0)),
            new_text: "  ".to_owned(),
        }]
    );
    let bytes = apply(source, &edits).unwrap();
    let applied = std::str::from_utf8(&bytes).unwrap();
    assert_eq!(applied, "<art>\n  <Button />\n</art>\n");
    assert_eq!(
        format_on_type(applied, "/Examples.vue", position, &options, VueVersion::V3),
        Some(Vec::new())
    );
    let editing = "<art>\n<Button   a=\"1\" />\n</art>\n";
    assert_eq!(
        format_on_type(editing, "/Examples.vue", position, &options, VueVersion::V3),
        Some(Vec::new())
    );
}

#[test]
fn invalid_block_boundaries_and_ranges_are_refused_without_partial_art_edits() {
    let source = "<art><Button /></art>\n<template>";
    let options = FormatOptions::default();
    assert_eq!(
        selected(
            source,
            Range::new(Position::new(0, 6), Position::new(0, 7)),
            &options
        ),
        None
    );
    let valid = "<art><Button   /></art>\n";
    assert_eq!(
        selected(
            valid,
            Range::new(Position::new(0, 6), Position::new(99, 0)),
            &options
        ),
        None
    );
}

#[test]
fn a_recoverable_script_error_does_not_block_or_rewrite_selected_art() {
    // Glyph preserves script parse failures verbatim; partial formatting must
    // retain that policy instead of rewriting or rejecting the unrelated script.
    let source = "<script setup>const value = ;</script>\n<art><Button /></art>\n";
    let edits = selected(
        source,
        Range::new(Position::new(1, 6), Position::new(1, 7)),
        &FormatOptions::default(),
    )
    .unwrap();
    assert_eq!(
        edits,
        vec![TextEdit {
            range: Range::new(Position::new(1, 5), Position::new(1, 15)),
            new_text: "\n  <Button />\n".to_owned(),
        }]
    );
    assert_eq!(
        std::str::from_utf8(&apply(source, &edits).unwrap()).unwrap(),
        "<script setup>const value = ;</script>\n<art>\n  <Button />\n</art>\n"
    );
}
