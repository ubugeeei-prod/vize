//! The same native CR position previously selected the following LF line.
#![expect(clippy::unwrap_used, reason = "complete native coordinate corpus")]

use super::{line_col_to_offset, offset_to_line_col};

const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/malformed-setup-metadata/valid-cr-import.vue.txt"
));

#[test]
fn real_cr_corpus_preserves_the_native_expression_error_position() {
    let broken = SOURCE.find("= ;").unwrap() as u32 + 2;
    assert_eq!(line_col_to_offset(SOURCE, 3, 15), Some(broken));
    assert_eq!(offset_to_line_col(SOURCE, broken), Some((3, 15)));
    assert_ne!(legacy_lf_offset(SOURCE, 3, 15), Some(broken));
}

#[test]
fn lsp_utf16_break_boundaries_preserve_unicode_and_refuse_invalid_columns() {
    for newline in ["\n", "\r", "\r\n"] {
        let source = vize_carton::cstr!("😀{newline}x{newline}");
        let second = 4 + newline.len() as u32;
        assert_eq!(line_col_to_offset(&source, 0, 1), None);
        assert_eq!(line_col_to_offset(&source, 0, 2), Some(4));
        assert_eq!(line_col_to_offset(&source, 0, 3), None);
        assert_eq!(line_col_to_offset(&source, 1, 0), Some(second));
        assert_eq!(offset_to_line_col(&source, second), Some((1, 0)));
        assert_eq!(line_col_to_offset(&source, 2, 0), Some(source.len() as u32));
        assert_eq!(line_col_to_offset(&source, 3, 0), None);
        assert_eq!(offset_to_line_col(&source, source.len() as u32 + 1), None);
    }
    let source = "😀\u{2028}\u{2029}\r\nx\ry";
    assert_eq!(line_col_to_offset(source, 0, 4), Some(10));
    assert_eq!(offset_to_line_col(source, 10), Some((0, 4)));
    assert_eq!(line_col_to_offset(source, 2, 1), Some(15));
    // Compiler LS/PS positions remain explicitly different from LSP positions.
    assert_eq!(
        vize_carton::line_index::LineBreaks::TypeScript.position_to_offset(source, 1, 0),
        Some(7)
    );
}

#[test]
fn public_virtual_project_mapping_keeps_actual_cr_and_utf16_positions() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().canonicalize().unwrap();
    let path = root.join("App.ts");
    let source = "const value = '😀';\rconst broken = ;\r\nvoid value;";
    std::fs::write(&path, source).unwrap();
    let mut project = crate::batch::virtual_project::VirtualProject::new(&root).unwrap();
    project
        .register_script_file(&path, source, oxc_span::SourceType::ts())
        .unwrap();
    let file = project.find_by_original(&path).unwrap();
    assert_eq!(file.content.as_str(), source);
    let virtual_path = file.virtual_path.clone();
    let original = project.map_to_original(&virtual_path, 1, 15).unwrap();
    assert_eq!(
        (&original.path, original.line, original.column),
        (&path, 1, 15)
    );
    assert_eq!(
        project.map_to_virtual(&path, 1, 15),
        Some((virtual_path.clone(), 1, 15))
    );
    assert!(project.map_to_original(&virtual_path, 0, 16).is_none());
    assert_eq!(project.map_to_virtual(&path, 0, 16), None);
    assert!(project.map_to_original(&virtual_path, 3, 0).is_none());
    assert_eq!(project.map_to_virtual(&path, 3, 0), None);
    assert_eq!(legacy_lf_offset(source, 1, 15), None);
}

/// Literal prior scalar/LF contract; only this test retains it as a before control.
fn legacy_lf_offset(content: &str, line: u32, col: u32) -> Option<u32> {
    let mut current_line = 0u32;
    let mut current_col = 0u32;
    let mut offset = 0u32;
    for ch in content.chars() {
        if current_line == line && current_col == col {
            return Some(offset);
        }
        if ch == '\n' {
            if current_line == line {
                return None;
            }
            current_line += 1;
            current_col = 0;
        } else {
            current_col += 1;
        }
        offset += ch.len_utf8() as u32;
    }
    if current_line == line && current_col == col {
        return Some(offset);
    }
    None
}
