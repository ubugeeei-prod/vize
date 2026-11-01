use super::{offset_to_line_col, ranges_overlap, template_position};
use tower_lsp::lsp_types::{Position, Range};

#[test]
fn test_ranges_overlap() {
    let a = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 10,
        },
    };
    let b = Range {
        start: Position {
            line: 0,
            character: 5,
        },
        end: Position {
            line: 0,
            character: 15,
        },
    };
    assert!(ranges_overlap(&a, &b));

    let c = Range {
        start: Position {
            line: 0,
            character: 20,
        },
        end: Position {
            line: 0,
            character: 30,
        },
    };
    assert!(!ranges_overlap(&a, &c));
}

#[test]
fn test_offset_to_line_col() {
    let source = "abc\ndef\nghi";
    assert_eq!(offset_to_line_col(source, 0), (0, 0));
    assert_eq!(offset_to_line_col(source, 3), (0, 3));
    assert_eq!(offset_to_line_col(source, 4), (1, 0));
    assert_eq!(offset_to_line_col(source, 8), (2, 0));
}

#[test]
fn offset_to_line_col_counts_utf16_code_units() {
    let source = r#"<div title="😀"  id="target"></div>"#;
    let offset = source.find("  id").unwrap();

    assert_eq!(offset_to_line_col(source, offset), (0, 15));
}

#[test]
fn template_position_maps_content_lines_to_lsp_lines() {
    assert_eq!(
        template_position("<template>\n<div title=\"hi\"  class=\"a\">", 10, 18),
        Position {
            line: 1,
            character: 17,
        }
    );
}

#[test]
fn test_get_line_indent() {
    use super::get_line_indent;

    assert_eq!(get_line_indent("hello", 0), "");
    assert_eq!(get_line_indent("  hello", 3), "  ");
    assert_eq!(get_line_indent("a\n  hello", 5), "  ");
    assert_eq!(get_line_indent("\t\thello", 3), "\t\t");
}

#[test]
fn test_diff_paths_relative_specifier() {
    use super::relative_specifier;
    let from = std::env::temp_dir().join("vize-ai-test-from");
    let target_dir = std::env::temp_dir().join("vize-ai-test-target");
    std::fs::create_dir_all(&from).unwrap();
    std::fs::create_dir_all(&target_dir).unwrap();
    let target = target_dir.join("MyButton.vue");
    std::fs::write(&target, "").unwrap();
    let result = relative_specifier(&from, &target).unwrap();
    assert!(
        result.contains("MyButton.vue"),
        "expected MyButton.vue in result, got {result:?}",
    );
    assert!(
        result.starts_with('.'),
        "expected leading `./` or `../`, got {result:?}",
    );
    std::fs::remove_dir_all(&from).ok();
    std::fs::remove_dir_all(&target_dir).ok();
}
