//! Code action provider for Vue SFC files.
//!
//! Provides code actions for:
//! - Lint fixes from vize_patina
//! - Quick fixes for common issues
//! - Refactoring actions
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use super::IdeContext;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Position, Range, TextEdit, WorkspaceEdit,
};

/// Code action service for providing quick fixes and refactorings.
pub struct CodeActionService;

impl CodeActionService {
    /// Get code actions for the given context and range.
    pub fn code_actions(ctx: &IdeContext, range: Range) -> Vec<CodeActionOrCommand> {
        let mut actions = Vec::new();

        // Collect lint fix actions
        actions.extend(Self::collect_lint_fixes(ctx, range));

        // Collect "@vize:forget" suppress actions
        actions.extend(Self::collect_forget_suppress(ctx, range));

        // Vue-flavored quick fixes: today this surfaces a "Wrap with `.value`"
        // edit when the cursor sits on a known reactive ref. See #691.
        actions.extend(Self::collect_wrap_with_value(ctx, range));

        actions
    }

    /// Offer a "Wrap with `.value`" quick fix when the cursor sits on an
    /// identifier that resolves to a reactive ref in script context. This is
    /// the most common Vue fix-up; complementary actions (unwrap, add to
    /// defineEmits, etc.) follow the same shape and will land in follow-ups.
    fn collect_wrap_with_value(ctx: &IdeContext, range: Range) -> Vec<CodeActionOrCommand> {
        // Only activate on script blocks — refs in template are auto-unwrapped.
        if !matches!(
            ctx.block_type,
            Some(crate::virtual_code::BlockType::ScriptSetup)
                | Some(crate::virtual_code::BlockType::Script)
        ) {
            return Vec::new();
        }

        let Some(identifier_range) = identifier_at_cursor(&ctx.content, ctx.offset) else {
            return Vec::new();
        };
        let identifier = &ctx.content[identifier_range.clone()];
        if !is_reactive_ref_in_script(&ctx.content, ctx.uri, identifier) {
            return Vec::new();
        }

        let (start_line, start_col) = offset_to_line_col(&ctx.content, identifier_range.end);
        let edit_range = Range {
            start: Position {
                line: start_line,
                character: start_col,
            },
            end: Position {
                line: start_line,
                character: start_col,
            },
        };

        let mut changes = std::collections::HashMap::new();
        changes.insert(
            ctx.uri.clone(),
            vec![TextEdit {
                range: edit_range,
                new_text: ".value".to_string(),
            }],
        );

        #[allow(clippy::disallowed_macros)]
        let action = CodeAction {
            title: format!("Wrap `{identifier}` with `.value`"),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: None,
            edit: Some(WorkspaceEdit {
                changes: Some(changes),
                document_changes: None,
                change_annotations: None,
            }),
            command: None,
            is_preferred: Some(true),
            disabled: None,
            data: None,
        };
        let _ = range;
        vec![CodeActionOrCommand::CodeAction(action)]
    }

    /// Collect lint fix actions from vize_patina diagnostics.
    fn collect_lint_fixes(ctx: &IdeContext, range: Range) -> Vec<CodeActionOrCommand> {
        let mut actions = Vec::new();

        // Parse SFC to get template
        #[allow(clippy::disallowed_methods)]
        let options = vize_atelier_sfc::SfcParseOptions {
            filename: ctx.uri.path().to_string().into(),
            ..Default::default()
        };

        let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&ctx.content, options) else {
            return actions;
        };

        let Some(ref template) = descriptor.template else {
            return actions;
        };

        // Run linter to get diagnostics with fixes
        let linter = vize_patina::Linter::new();
        let result = linter.lint_template(&template.content, ctx.uri.path());

        // Template block offset in SFC
        let template_start_line = template.loc.start_line as u32;

        for lint_diag in result.diagnostics {
            // Check if diagnostic has a fix
            let Some(ref fix) = lint_diag.fix else {
                continue;
            };

            // Convert lint diagnostic position to SFC position
            let (start_line, start_col) =
                offset_to_line_col(&template.content, lint_diag.start as usize);
            let (end_line, end_col) = offset_to_line_col(&template.content, lint_diag.end as usize);

            let diag_range = Range {
                start: template_position(template_start_line, start_line, start_col),
                end: template_position(template_start_line, end_line, end_col),
            };

            // Check if the diagnostic range overlaps with the requested range
            if !ranges_overlap(&diag_range, &range) {
                continue;
            }

            // Convert fix edits to LSP TextEdits
            let edits: Vec<TextEdit> = fix
                .edits
                .iter()
                .map(|edit| {
                    let (edit_start_line, edit_start_col) =
                        offset_to_line_col(&template.content, edit.start as usize);
                    let (edit_end_line, edit_end_col) =
                        offset_to_line_col(&template.content, edit.end as usize);

                    TextEdit {
                        range: Range {
                            start: template_position(
                                template_start_line,
                                edit_start_line,
                                edit_start_col,
                            ),
                            end: template_position(
                                template_start_line,
                                edit_end_line,
                                edit_end_col,
                            ),
                        },
                        #[allow(clippy::disallowed_methods)]
                        new_text: edit.new_text.to_string(),
                    }
                })
                .collect();

            // Create workspace edit
            #[allow(clippy::disallowed_types)]
            let mut changes = std::collections::HashMap::new();
            changes.insert(ctx.uri.clone(), edits);

            let workspace_edit = WorkspaceEdit {
                changes: Some(changes),
                document_changes: None,
                change_annotations: None,
            };

            // Create code action
            #[allow(clippy::disallowed_macros)]
            let action = CodeAction {
                title: format!("Fix: {}", fix.message),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: None, // Could link to specific diagnostic
                edit: Some(workspace_edit),
                command: None,
                is_preferred: Some(true),
                disabled: None,
                data: None,
            };

            actions.push(CodeActionOrCommand::CodeAction(action));
        }

        actions
    }

    /// Collect `@vize:forget` suppress actions for diagnostics without auto-fix.
    fn collect_forget_suppress(ctx: &IdeContext, range: Range) -> Vec<CodeActionOrCommand> {
        let mut actions = Vec::new();

        #[allow(clippy::disallowed_methods)]
        let options = vize_atelier_sfc::SfcParseOptions {
            filename: ctx.uri.path().to_string().into(),
            ..Default::default()
        };

        let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&ctx.content, options) else {
            return actions;
        };

        let Some(ref template) = descriptor.template else {
            return actions;
        };

        let linter = vize_patina::Linter::new();
        let result = linter.lint_template(&template.content, ctx.uri.path());

        let template_start_line = template.loc.start_line as u32;

        for lint_diag in result.diagnostics {
            // Convert diagnostic position to SFC position
            let (start_line, start_col) =
                offset_to_line_col(&template.content, lint_diag.start as usize);
            let (end_line, end_col) = offset_to_line_col(&template.content, lint_diag.end as usize);

            let diag_range = Range {
                start: template_position(template_start_line, start_line, start_col),
                end: template_position(template_start_line, end_line, end_col),
            };

            if !ranges_overlap(&diag_range, &range) {
                continue;
            }

            // Compute indentation of the diagnostic line
            let indent = get_line_indent(&template.content, lint_diag.start as usize);

            // Insert `<!-- @vize:forget <rule_name> -->\n` before the line
            let sfc_line = template_position(template_start_line, start_line, 0).line;
            let insert_pos = Position {
                line: sfc_line,
                character: 0,
            };

            #[allow(clippy::disallowed_macros)]
            let new_text = format!("{}<!-- @vize:forget {} -->\n", indent, lint_diag.rule_name,);

            let edit = TextEdit {
                range: Range {
                    start: insert_pos,
                    end: insert_pos,
                },
                new_text,
            };

            #[allow(clippy::disallowed_types)]
            let mut changes = std::collections::HashMap::new();
            changes.insert(ctx.uri.clone(), vec![edit]);

            let workspace_edit = WorkspaceEdit {
                changes: Some(changes),
                document_changes: None,
                change_annotations: None,
            };

            #[allow(clippy::disallowed_macros)]
            let action = CodeAction {
                title: format!("Suppress with @vize:forget ({})", lint_diag.rule_name),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: None,
                edit: Some(workspace_edit),
                command: None,
                is_preferred: Some(false),
                disabled: None,
                data: None,
            };

            actions.push(CodeActionOrCommand::CodeAction(action));
        }

        actions
    }

    /// Get all available fixes for a document (for "fix all" actions).
    pub fn get_all_fixes(ctx: &IdeContext) -> Option<WorkspaceEdit> {
        #[allow(clippy::disallowed_methods)]
        let options = vize_atelier_sfc::SfcParseOptions {
            filename: ctx.uri.path().to_string().into(),
            ..Default::default()
        };

        let descriptor = vize_atelier_sfc::parse_sfc(&ctx.content, options).ok()?;
        let template = descriptor.template.as_ref()?;

        let linter = vize_patina::Linter::new();
        let result = linter.lint_template(&template.content, ctx.uri.path());

        let template_start_line = template.loc.start_line as u32;

        let mut all_edits: Vec<TextEdit> = Vec::new();

        for lint_diag in result.diagnostics {
            if let Some(ref fix) = lint_diag.fix {
                for edit in &fix.edits {
                    let (edit_start_line, edit_start_col) =
                        offset_to_line_col(&template.content, edit.start as usize);
                    let (edit_end_line, edit_end_col) =
                        offset_to_line_col(&template.content, edit.end as usize);

                    all_edits.push(TextEdit {
                        range: Range {
                            start: template_position(
                                template_start_line,
                                edit_start_line,
                                edit_start_col,
                            ),
                            end: template_position(
                                template_start_line,
                                edit_end_line,
                                edit_end_col,
                            ),
                        },
                        #[allow(clippy::disallowed_methods)]
                        new_text: edit.new_text.to_string(),
                    });
                }
            }
        }

        if all_edits.is_empty() {
            return None;
        }

        // Sort edits by position (reverse order for safe application)
        all_edits.sort_by(|a, b| {
            b.range
                .start
                .line
                .cmp(&a.range.start.line)
                .then(b.range.start.character.cmp(&a.range.start.character))
        });

        // Remove overlapping edits (keep the first one)
        let mut filtered_edits: Vec<TextEdit> = Vec::new();
        for edit in all_edits {
            let overlaps = filtered_edits
                .iter()
                .any(|e| ranges_overlap(&e.range, &edit.range));
            if !overlaps {
                filtered_edits.push(edit);
            }
        }

        #[allow(clippy::disallowed_types)]
        let mut changes = std::collections::HashMap::new();
        changes.insert(ctx.uri.clone(), filtered_edits);

        Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        })
    }
}

/// Convert byte offset to (line, column) - both 0-indexed for LSP.
fn offset_to_line_col(source: &str, offset: usize) -> (u32, u32) {
    let mut line = 0u32;
    let mut col = 0u32;
    let mut current_offset = 0;

    for ch in source.chars() {
        if current_offset >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += ch.len_utf16() as u32;
        }
        current_offset += ch.len_utf8();
    }

    (line, col)
}

fn template_position(template_start_line: u32, line: u32, character: u32) -> Position {
    Position {
        line: template_start_line.saturating_sub(1) + line,
        character,
    }
}

/// Get the leading whitespace (indentation) for the line containing the given byte offset.
fn get_line_indent(source: &str, offset: usize) -> &str {
    let bytes = source.as_bytes();

    // Find the start of the line
    let line_start = if offset == 0 {
        0
    } else {
        bytes[..offset]
            .iter()
            .rposition(|&b| b == b'\n')
            .map_or(0, |pos| pos + 1)
    };

    // Collect whitespace from the start of the line
    let rest = &source[line_start..];
    let indent_len = rest
        .bytes()
        .take_while(|b| *b == b' ' || *b == b'\t')
        .count();

    &source[line_start..line_start + indent_len]
}

/// Check if two ranges overlap.
/// Return the byte range of the identifier under the cursor, or `None` when
/// the cursor is not on an identifier.
fn identifier_at_cursor(content: &str, offset: usize) -> Option<std::ops::Range<usize>> {
    let bytes = content.as_bytes();
    let end = offset.min(content.len());
    let mut start = end;
    while start > 0 && is_ident_byte(bytes[start - 1]) {
        start -= 1;
    }
    let mut walk = end;
    while walk < bytes.len() && is_ident_byte(bytes[walk]) {
        walk += 1;
    }
    if start == walk {
        return None;
    }
    Some(start..walk)
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

/// True when `name` resolves to a Vue ref (Ref / ShallowRef / ToRef /
/// ComputedRef) declared anywhere in the SFC's script-setup block. Used to
/// decide whether "Wrap with `.value`" makes sense at the cursor.
fn is_reactive_ref_in_script(content: &str, uri: &tower_lsp::lsp_types::Url, name: &str) -> bool {
    use vize_croquis::reactivity::ReactiveKind;
    let options = vize_atelier_sfc::SfcParseOptions {
        filename: uri.path().to_string().into(),
        ..Default::default()
    };
    let Ok(descriptor) = vize_atelier_sfc::parse_sfc(content, options) else {
        return false;
    };
    let Some(ref script_setup) = descriptor.script_setup else {
        return false;
    };
    let mut analyzer = vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions {
        analyze_script: true,
        ..Default::default()
    });
    analyzer.analyze_script_setup(&script_setup.content);
    let croquis = analyzer.finish();
    croquis.reactivity.lookup(name).is_some_and(|source| {
        matches!(
            source.kind,
            ReactiveKind::Ref
                | ReactiveKind::ShallowRef
                | ReactiveKind::ToRef
                | ReactiveKind::Computed
        )
    })
}

fn ranges_overlap(a: &Range, b: &Range) -> bool {
    // Ranges overlap if neither is completely before or after the other
    !(a.end.line < b.start.line
        || (a.end.line == b.start.line && a.end.character < b.start.character)
        || b.end.line < a.start.line
        || (b.end.line == a.start.line && b.end.character < a.start.character))
}

#[cfg(test)]
mod tests {
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
            template_position(1, 1, 17),
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
}
