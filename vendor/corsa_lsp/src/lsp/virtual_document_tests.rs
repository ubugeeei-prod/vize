use super::{VirtualChange, VirtualDocument};
use lsp_types::{Position, Range};

#[test]
fn utf16_ranges_apply_without_splitting_code_points() {
    let mut document =
        VirtualDocument::untitled("/virtual/demo.ts", "typescript", "const x = 😀;\n").unwrap();
    document
        .apply_changes(&[VirtualChange::splice(
            Range::new(Position::new(0, 10), Position::new(0, 12)),
            "42",
        )])
        .unwrap();
    assert_eq!(document.text, "const x = 42;\n");
    assert_eq!(document.version, 2);
}

#[test]
fn full_replacements_reset_the_buffer() {
    let mut document =
        VirtualDocument::in_memory("overlay", "/main.ts", "typescript", "let value = 1;").unwrap();
    document
        .apply_changes(&[VirtualChange::replace("let value = 2;")])
        .unwrap();
    assert_eq!(document.text, "let value = 2;");
    assert_eq!(document.version, 2);
}

#[test]
fn untitled_normalizes_relative_paths() {
    let document = VirtualDocument::untitled("virtual/demo.ts", "typescript", "").unwrap();
    assert_eq!(document.uri.as_str(), "untitled:/virtual/demo.ts");
    assert_eq!(document.key(), "untitled:/virtual/demo.ts");
}

#[test]
fn in_memory_normalizes_relative_paths() {
    let document = VirtualDocument::in_memory("overlay", "demo.ts", "typescript", "").unwrap();
    assert_eq!(document.uri.as_str(), "corsa://overlay/demo.ts");
}

#[test]
fn empty_change_batches_do_not_advance_version() {
    let mut document = VirtualDocument::untitled("/virtual/demo.ts", "typescript", "x").unwrap();
    let events = document.apply_changes(&[]).unwrap();
    assert!(events.is_empty());
    assert_eq!(document.version, 1);
    assert_eq!(document.text, "x");
}

#[test]
fn inverted_ranges_fail_without_mutating_the_document() {
    let mut document = VirtualDocument::untitled("/virtual/demo.ts", "typescript", "abc").unwrap();
    let err = document
        .apply_changes(&[VirtualChange::splice(
            Range::new(Position::new(0, 2), Position::new(0, 1)),
            "z",
        )])
        .unwrap_err();
    assert!(
        matches!(err, corsa_core::CorsaError::Protocol(message) if message.contains("inverted range"))
    );
    assert_eq!(document.version, 1);
    assert_eq!(document.text, "abc");
}

#[test]
fn out_of_bounds_ranges_fail_without_mutating_the_document() {
    let mut document = VirtualDocument::untitled("/virtual/demo.ts", "typescript", "abc").unwrap();
    let err = document
        .apply_changes(&[VirtualChange::splice(
            Range::new(Position::new(1, 0), Position::new(1, 0)),
            "z",
        )])
        .unwrap_err();
    assert!(
        matches!(err, corsa_core::CorsaError::Protocol(message) if message.contains("out of bounds"))
    );
    assert_eq!(document.version, 1);
    assert_eq!(document.text, "abc");
}

#[test]
fn splitting_a_code_point_is_rejected() {
    let mut document = VirtualDocument::untitled("/virtual/demo.ts", "typescript", "😀").unwrap();
    let err = document
        .apply_changes(&[VirtualChange::splice(
            Range::new(Position::new(0, 1), Position::new(0, 1)),
            "x",
        )])
        .unwrap_err();
    assert!(
        matches!(err, corsa_core::CorsaError::Protocol(message) if message.contains("splits a code point"))
    );
    assert_eq!(document.version, 1);
    assert_eq!(document.text, "😀");
}
