//! Document formatting support.
//!
//! Provides SFC document formatting via the vize_glyph formatter.

#[cfg(feature = "glyph")]
use tower_lsp::lsp_types::{Position, Range, TextEdit};

/// Format a document and return TextEdits for the LSP client.
///
/// Returns `Some(vec![])` if no changes needed, `Some(vec![edit])` with the
/// full-document replacement, or `None` on formatting error.
#[cfg(feature = "glyph")]
pub(crate) fn format_document(
    content: &str,
    options: &vize_glyph::FormatOptions,
) -> Option<Vec<TextEdit>> {
    let allocator = vize_glyph::Allocator::with_capacity(content.len());

    let formatted = match vize_glyph::format_sfc_with_allocator(content, options, &allocator) {
        Ok(result) => result,
        Err(_) => return None,
    };

    if !formatted.changed {
        return Some(vec![]);
    }

    let line_count = content.lines().count() as u32;
    let last_line_len = content.lines().last().map_or(0, |l| l.len()) as u32;
    Some(vec![TextEdit {
        range: Range {
            start: Position::new(0, 0),
            end: Position::new(line_count, last_line_len),
        },
        #[allow(clippy::disallowed_methods)]
        new_text: formatted.code.to_string(),
    }])
}

#[cfg(all(test, feature = "glyph"))]
mod tests {
    use super::format_document;
    use crate::server::ServerState;
    use tower_lsp::lsp_types::Position;

    #[test]
    fn format_document_is_idempotent() {
        let source = "<template>\n<div>hello</div>\n</template>\n";
        let options = vize_glyph::FormatOptions::default();

        let result = format_document(source, &options);
        assert!(result.is_some());
        let edits = result.unwrap();
        assert!(!edits.is_empty(), "expected edits on first format");

        let formatted = &edits[0].new_text;
        let result2 = format_document(formatted, &options);
        assert!(result2.is_some());
        let edits2 = result2.unwrap();
        assert!(
            edits2.is_empty(),
            "expected no edits on second format (idempotent)"
        );
    }

    #[test]
    fn format_document_returns_edit_for_unformatted() {
        let source = "<template>\n<div>hello</div>\n</template>\n";
        let options = vize_glyph::FormatOptions::default();
        let result = format_document(source, &options);
        assert!(result.is_some());
        let edits = result.unwrap();
        if !edits.is_empty() {
            assert_eq!(edits.len(), 1);
            let edit = &edits[0];
            assert_eq!(edit.range.start, Position::new(0, 0));
            insta::assert_debug_snapshot!(edits);
        }
    }

    #[test]
    fn format_document_respects_options() {
        let source = "<script>\nconst x = 1;\n</script>\n";
        let options = vize_glyph::FormatOptions {
            semi: false,
            ..Default::default()
        };
        let result = format_document(source, &options);
        assert!(result.is_some());
        let edits = result.unwrap();
        if !edits.is_empty() {
            insta::assert_snapshot!(edits[0].new_text.as_str());
        }
    }

    #[test]
    fn format_document_edit_covers_full_range() {
        let source = "<template>\n<div   class=\"a\"   id=\"b\" >\nhello\n</div>\n</template>\n";
        let options = vize_glyph::FormatOptions::default();
        let result = format_document(source, &options);
        assert!(result.is_some());
        let edits = result.unwrap();
        if !edits.is_empty() {
            let edit = &edits[0];
            assert_eq!(edit.range.start, Position::new(0, 0));
            let line_count = source.lines().count() as u32;
            assert_eq!(edit.range.end.line, line_count);
        }
    }

    #[test]
    fn format_document_with_single_quote() {
        let source = "<script>\nconst x = \"hello\";\n</script>\n";
        let options = vize_glyph::FormatOptions {
            single_quote: true,
            ..Default::default()
        };
        let result = format_document(source, &options);
        assert!(result.is_some());
        let edits = result.unwrap();
        if !edits.is_empty() {
            insta::assert_debug_snapshot!(edits);
        }
    }

    #[test]
    fn format_document_with_config_loaded_from_state() {
        let state = ServerState::new();
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("vize.config.json"),
            r#"{ "fmt": { "singleQuote": true } }"#,
        )
        .unwrap();
        state.load_format_config(dir.path());

        let options = state.get_format_options();
        assert!(options.single_quote);

        let source = "<script>\nconst x = \"hello\";\n</script>\n";
        let result = format_document(source, &options);
        assert!(result.is_some());
        let edits = result.unwrap();
        if !edits.is_empty() {
            insta::assert_debug_snapshot!(edits);
        }
    }
}
