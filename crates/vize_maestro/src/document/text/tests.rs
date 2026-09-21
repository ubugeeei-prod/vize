use super::DocumentText;
use crate::utils::position::{line_range, offset_to_position, position_to_offset};
use tower_lsp::lsp_types::Position;
use vize_s0::line_index::LineBreaks;

fn assert_matches_lsp(text: &DocumentText, source: &str) {
    assert_eq!(vize_s0::cstr!("{text}"), source);
    let starts: Vec<_> = LineBreaks::Lsp.line_starts(source).collect();
    assert_eq!(text.len_lines(), starts.len());
    assert_eq!(text.line_to_byte(text.len_lines()), source.len());
    for (line, &start) in starts.iter().enumerate() {
        assert_eq!(text.line_to_byte(line), start);
        let end = starts.get(line + 1).copied().unwrap_or(source.len());
        let rope_line = text.line(line);
        assert_eq!(vize_s0::cstr!("{rope_line}"), &source[start..end]);
        let content = source[start..end].trim_end_matches(['\r', '\n']);
        let units = content.encode_utf16().count() as u32;
        assert_eq!(
            line_range(text, line).unwrap().end,
            Position::new(line as u32, units)
        );
        for column in 0..=units + 1 {
            assert_eq!(
                position_to_offset(text, Position::new(line as u32, column)),
                LineBreaks::Lsp.position_to_offset(source, line as u32, column),
                "{source:?}, line {line}, column {column}"
            );
        }
    }
    for (offset, _) in source
        .char_indices()
        .chain(std::iter::once((source.len(), '\0')))
    {
        let (line, column) = LineBreaks::Lsp.offset_to_position(source, offset);
        assert_eq!(
            offset_to_position(text, offset),
            Some(Position::new(line, column))
        );
    }
}

#[test]
fn rope_and_string_coordinates_agree_for_all_line_break_conventions() {
    let pieces = [
        "", "a😀", "©€—", "\r", "\n", "\r\n", "\u{b}", "\u{c}", "\u{85}", "\u{2028}", "\u{2029}",
    ];
    for left in pieces {
        for right in pieces {
            let source = vize_s0::cstr!("{left}{right}é");
            assert_matches_lsp(&DocumentText::new(&source), &source);
        }
    }
}

#[test]
fn replacements_keep_exceptional_boundaries_attached_to_the_text() {
    let source = "a\u{2028}😀\r\n\u{85}z\u{b}\u{c}\u{2029}";
    let original = DocumentText::new(source);
    let boundaries: Vec<_> = source
        .char_indices()
        .map(|(at, _)| at)
        .chain(std::iter::once(source.len()))
        .collect();
    for start in 0..boundaries.len() {
        for end in start..boundaries.len() {
            for inserted in ["", "X", "\r", "\n", "\u{2028}Q\u{85}"] {
                let mut text = original.clone();
                text.replace(start..end, inserted);
                let expected = vize_s0::cstr!(
                    "{}{}{}",
                    &source[..boundaries[start]],
                    inserted,
                    &source[boundaries[end]..]
                );
                assert_matches_lsp(&text, &expected);
            }
        }
    }
    assert_matches_lsp(&original, source);
}

#[test]
fn joining_and_splitting_crlf_recomputes_the_exceptional_lines() {
    let mut text = DocumentText::new("a\r\u{2028}\nb\u{2029}c");
    text.replace(2..3, "");
    assert_matches_lsp(&text, "a\r\nb\u{2029}c");
    text.replace(2..2, "\u{85}");
    assert_matches_lsp(&text, "a\r\u{85}\nb\u{2029}c");
    text.replace(0..7, "normal\ntext");
    assert_matches_lsp(&text, "normal\ntext");
    assert!(text.ignored_breaks.is_empty());
}

#[test]
fn lsp_lines_can_span_several_rope_chunks() {
    let prefix = "x".repeat(4096);
    let source = vize_s0::cstr!("{prefix}\u{2028}{prefix}\u{85}😀\r\nlast");
    let text = DocumentText::new(&source);
    assert_eq!(text.len_lines(), 2);
    assert_eq!(
        position_to_offset(&text, Position::new(0, 8196)),
        Some(source.len() - 6)
    );
    assert_eq!(
        offset_to_position(&text, source.len() - 4),
        Some(Position::new(1, 0))
    );
    assert_eq!(line_range(&text, 0).unwrap().end, Position::new(0, 8196));
}

#[test]
fn ordinary_documents_need_no_exceptional_boundary_allocation() {
    let mut text = DocumentText::new("a😀\r\nb\nc");
    assert!(!text.ignored_breaks.spilled());
    text.replace(0..1, "hello\r\nworld");
    assert_matches_lsp(&text, "hello\r\nworld😀\r\nb\nc");
    assert!(text.ignored_breaks.is_empty());
    assert!(!text.ignored_breaks.spilled());
}

#[test]
fn snapshot_comparisons_follow_edits_without_flattening_the_rope() {
    let prefix = "x".repeat(4096);
    let source = vize_s0::cstr!("{prefix}\u{2028}😀\r\nlast");
    let mut text = DocumentText::new(&source);
    assert!(text == source.as_str());
    assert!(text != &source[..4096]);
    text.replace(4097..4098, "😃");
    assert!(text != source.as_str());
    let edited = vize_s0::cstr!("{prefix}\u{2028}😃\r\nlast");
    assert!(text == edited.as_str());
}
