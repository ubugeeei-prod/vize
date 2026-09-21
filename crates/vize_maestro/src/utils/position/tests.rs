//! Position conversion regression tests.
use super::{
    offset_to_position, offset_to_position_str, position_to_offset, position_to_offset_str,
};
use crate::document::DocumentText;
use tower_lsp::lsp_types::Position;

#[test]
fn rope_positions_reject_line_terminators_and_use_utf16_ranges() {
    for ending in ["\n", "\r", "\r\n"] {
        let source = vize_s0::cstr!("a😀{ending}b");
        let rope = DocumentText::new(&source);
        assert_eq!(position_to_offset(&rope, Position::new(0, 3)), Some(5));
        assert_eq!(position_to_offset(&rope, Position::new(0, 2)), None);
        assert_eq!(position_to_offset(&rope, Position::new(0, 4)), None);
        assert_eq!(
            position_to_offset(&rope, Position::new(1, 1)),
            Some(source.len())
        );
        assert_eq!(
            super::line_range(&rope, 0).unwrap().end,
            Position::new(0, 3)
        );
    }
}

#[test]
fn rope_utf16_tree_measures_cross_many_chunks() {
    let source = vize_s0::cstr!("prefix\n{}x\n", "a😀".repeat(8192));
    let rope = DocumentText::new(&source);
    let end = source.len() - 1;
    assert_eq!(
        offset_to_position(&rope, end),
        Some(Position::new(1, 8192 * 3 + 1))
    );
    assert_eq!(
        position_to_offset(&rope, Position::new(1, 8192 * 3 + 1)),
        Some(end)
    );
    assert_eq!(
        position_to_offset(&rope, Position::new(1, 8192 * 3 - 1)),
        None
    );
}

#[test]
fn test_offset_to_position() {
    let rope = DocumentText::new("hello\nworld\n");

    // Start of file
    assert_eq!(
        offset_to_position(&rope, 0),
        Some(Position {
            line: 0,
            character: 0,
        })
    );

    // Middle of first line
    assert_eq!(
        offset_to_position(&rope, 3),
        Some(Position {
            line: 0,
            character: 3,
        })
    );

    // Start of second line
    assert_eq!(
        offset_to_position(&rope, 6),
        Some(Position {
            line: 1,
            character: 0,
        })
    );

    // End of file
    assert_eq!(
        offset_to_position(&rope, 12),
        Some(Position {
            line: 2,
            character: 0,
        })
    );
}

#[test]
fn test_position_to_offset() {
    let rope = DocumentText::new("hello\nworld\n");

    // Start of file
    assert_eq!(
        position_to_offset(
            &rope,
            Position {
                line: 0,
                character: 0,
            }
        ),
        Some(0)
    );

    // Middle of first line
    assert_eq!(
        position_to_offset(
            &rope,
            Position {
                line: 0,
                character: 3,
            }
        ),
        Some(3)
    );

    // Start of second line
    assert_eq!(
        position_to_offset(
            &rope,
            Position {
                line: 1,
                character: 0,
            }
        ),
        Some(6)
    );
}

#[test]
fn test_offset_to_position_counts_utf16_code_units() {
    let rope = DocumentText::new("a😀b\nc");

    assert_eq!(
        offset_to_position(&rope, "a😀".len()),
        Some(Position {
            line: 0,
            character: 3,
        })
    );
    assert_eq!(
        offset_to_position(&rope, "a😀b\nc".len()),
        Some(Position {
            line: 1,
            character: 1,
        })
    );
}

#[test]
fn test_position_to_offset_counts_utf16_code_units() {
    let rope = DocumentText::new("a😀b\nc");

    assert_eq!(
        position_to_offset(
            &rope,
            Position {
                line: 0,
                character: 3,
            }
        ),
        Some("a😀".len())
    );
    assert_eq!(
        position_to_offset(
            &rope,
            Position {
                line: 0,
                character: 4,
            }
        ),
        Some("a😀b".len())
    );
}

#[test]
fn test_position_to_offset_rejects_utf16_surrogate_pair_interior() {
    let rope = DocumentText::new("a😀b");

    assert_eq!(
        position_to_offset(
            &rope,
            Position {
                line: 0,
                character: 2,
            }
        ),
        None
    );
}

#[test]
fn test_offset_to_position_str_counts_utf16_code_units() {
    let content = "const icon = \"😀\";\nconst message = icon";
    let message_offset = content.find("message").unwrap();

    assert_eq!(
        offset_to_position_str(content, message_offset),
        Position {
            line: 1,
            character: 6,
        }
    );

    assert_eq!(
        offset_to_position_str(content, "const icon = \"😀".len()),
        Position {
            line: 0,
            character: 16,
        }
    );
}

#[test]
fn test_position_to_offset_str_counts_utf16_code_units() {
    let content = "a😀b\nc";

    assert_eq!(position_to_offset_str(content, 0, 3), "a😀".len());
    assert_eq!(position_to_offset_str(content, 0, 4), "a😀b".len());
    assert_eq!(position_to_offset_str(content, 1, 1), content.len());
}

#[test]
fn string_positions_follow_lsp_breaks_and_utf16_columns() {
    let source = "a\r\u{1f600}\u{2028}x\r\ny";
    assert_eq!(
        offset_to_position_str(source, source.find('x').unwrap()),
        Position::new(1, 3)
    );
    assert_eq!(
        position_to_offset_str(source, 1, 3),
        source.find('x').unwrap()
    );
    assert_eq!(
        position_to_offset_str(source, 1, 99),
        source.find("\r\n").unwrap()
    );
    assert_eq!(position_to_offset_str(source, 2, 0), source.len() - 1);
}
