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
        actions.extend(Self::collect_unwrap_value(ctx, range));
        actions.extend(Self::collect_auto_import(ctx, range));

        actions
    }

    /// Offer "Auto-import from <path>" when the cursor sits on an identifier
    /// that the current SFC's script doesn't declare, but the workspace
    /// index built in #722 knows about.
    fn collect_auto_import(ctx: &IdeContext, range: Range) -> Vec<CodeActionOrCommand> {
        let _ = range;
        let Some(identifier_range) = identifier_at_cursor(&ctx.content, ctx.offset) else {
            return Vec::new();
        };
        let identifier = &ctx.content[identifier_range.clone()];
        if identifier.is_empty()
            || identifier
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_digit())
        {
            return Vec::new();
        }

        // Skip names that the SFC already binds. Looking only at script
        // setup is sufficient for the common case; the broader Croquis
        // resolution moves in a follow-up.
        if sfc_script_declares(&ctx.content, identifier) {
            return Vec::new();
        }

        // Index the directory the current file lives in. Recursive scan
        // and tsconfig path-alias resolution are tracked in #689; the
        // single-directory scan is enough to demonstrate end-to-end value
        // and keeps the action responsive on large workspaces.
        let workspace_dir = ctx
            .uri
            .to_file_path()
            .ok()
            .and_then(|path| path.parent().map(|p| p.to_path_buf()));
        let Some(workspace_dir) = workspace_dir else {
            return Vec::new();
        };
        let index = crate::ide::auto_import::AutoImportIndex::from_directory(&workspace_dir);
        let entry = index.lookup(identifier).next().cloned();
        let Some(entry) = entry else {
            return Vec::new();
        };

        let import_specifier = match relative_specifier(&workspace_dir, &entry.source) {
            Some(s) => s,
            None => return Vec::new(),
        };
        #[allow(clippy::disallowed_macros)]
        let import_statement = match entry.kind {
            crate::ide::auto_import::AutoImportKind::VueComponent => {
                format!("import {} from '{}'\n", entry.name, import_specifier)
            }
            crate::ide::auto_import::AutoImportKind::Composable => {
                format!("import {{ {} }} from '{}'\n", entry.name, import_specifier)
            }
        };

        // Insert at the start of script setup if present, otherwise at the
        // start of the regular script block.
        let options = vize_atelier_sfc::SfcParseOptions {
            filename: ctx.uri.path().to_string().into(),
            ..Default::default()
        };
        let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&ctx.content, options) else {
            return Vec::new();
        };
        let insert_offset = descriptor
            .script_setup
            .as_ref()
            .map(|s| s.loc.start)
            .or_else(|| descriptor.script.as_ref().map(|s| s.loc.start));
        let Some(insert_offset) = insert_offset else {
            return Vec::new();
        };
        let (line, character) = offset_to_line_col(&ctx.content, insert_offset);
        let insert_position = Position { line, character };
        let edit_range = Range {
            start: insert_position,
            end: insert_position,
        };

        let mut changes = std::collections::HashMap::new();
        changes.insert(
            ctx.uri.clone(),
            vec![TextEdit {
                range: edit_range,
                new_text: import_statement,
            }],
        );

        #[allow(clippy::disallowed_macros)]
        let action = CodeAction {
            title: format!("Auto-import `{}` from `{}`", entry.name, import_specifier),
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
        vec![CodeActionOrCommand::CodeAction(action)]
    }

    /// Offer an "Unwrap `.value`" quick fix when the cursor sits on the
    /// `.value` segment of a reactive ref access in script context. Useful
    /// when the user copies template code (where refs auto-unwrap) into
    /// script and the trailing `.value` is now redundant — or when they
    /// pass `count.value` to a function that expects the wrapper itself.
    fn collect_unwrap_value(ctx: &IdeContext, range: Range) -> Vec<CodeActionOrCommand> {
        if !matches!(
            ctx.block_type,
            Some(crate::virtual_code::BlockType::ScriptSetup)
                | Some(crate::virtual_code::BlockType::Script)
        ) {
            return Vec::new();
        }

        // Look for a `.value` token straddling the cursor.
        let cursor = ctx.offset.min(ctx.content.len());
        let before = &ctx.content[..cursor];
        let after = &ctx.content[cursor..];
        let dot_pos = match (before.rfind(".value"), after.starts_with("value")) {
            (Some(p), _) if cursor >= p && cursor <= p + ".value".len() => p,
            (Some(p), _) if cursor == p + ".value".len() => p,
            _ if before.ends_with('.') && after.starts_with("value") => cursor - 1,
            _ => return Vec::new(),
        };
        if !before[..dot_pos].ends_with(|c: char| c.is_alphanumeric() || c == '_' || c == '$') {
            return Vec::new();
        }
        // Resolve the receiver identifier just before the `.`.
        let bytes = ctx.content.as_bytes();
        let mut start = dot_pos;
        while start > 0 && is_ident_byte(bytes[start - 1]) {
            start -= 1;
        }
        let receiver = &ctx.content[start..dot_pos];
        if !is_reactive_ref_in_script(&ctx.content, ctx.uri, receiver) {
            return Vec::new();
        }

        let value_end = dot_pos + ".value".len();
        let (start_line, start_col) = offset_to_line_col(&ctx.content, dot_pos);
        let (end_line, end_col) = offset_to_line_col(&ctx.content, value_end);
        let edit_range = Range {
            start: Position {
                line: start_line,
                character: start_col,
            },
            end: Position {
                line: end_line,
                character: end_col,
            },
        };

        let mut changes = std::collections::HashMap::new();
        changes.insert(
            ctx.uri.clone(),
            vec![TextEdit {
                range: edit_range,
                new_text: String::new(),
            }],
        );

        #[allow(clippy::disallowed_macros)]
        let action = CodeAction {
            title: format!("Unwrap `.value` from `{receiver}`"),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: None,
            edit: Some(WorkspaceEdit {
                changes: Some(changes),
                document_changes: None,
                change_annotations: None,
            }),
            command: None,
            is_preferred: None,
            disabled: None,
            data: None,
        };
        let _ = range;
        vec![CodeActionOrCommand::CodeAction(action)]
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
/// True when the SFC's script (setup or plain) declares `name` via Croquis'
/// binding analysis. Used by the auto-import code action to skip names the
/// user already imported / declared.
fn sfc_script_declares(content: &str, name: &str) -> bool {
    let options = vize_atelier_sfc::SfcParseOptions::default();
    let Ok(descriptor) = vize_atelier_sfc::parse_sfc(content, options) else {
        return false;
    };
    let script_content = descriptor
        .script_setup
        .as_ref()
        .map(|s| s.content.to_string())
        .or_else(|| descriptor.script.as_ref().map(|s| s.content.to_string()));
    let Some(script_content) = script_content else {
        return false;
    };
    let mut analyzer = vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions {
        analyze_script: true,
        ..Default::default()
    });
    if descriptor.script_setup.is_some() {
        analyzer.analyze_script_setup(&script_content);
    } else {
        analyzer.analyze_script_plain(&script_content);
    }
    analyzer.finish().bindings.contains(name)
}

/// Convert an absolute path to a relative module specifier suitable for an
/// import statement (`./Button.vue`, `../shared/useCounter`). Returns `None`
/// when the two paths share no common root.
fn relative_specifier(from_dir: &std::path::Path, target: &std::path::Path) -> Option<String> {
    let target = target
        .canonicalize()
        .unwrap_or_else(|_| target.to_path_buf());
    let from_dir = from_dir
        .canonicalize()
        .unwrap_or_else(|_| from_dir.to_path_buf());
    let rel = diff_paths(&target, &from_dir)?;
    let mut s = rel.to_string_lossy().to_string();
    if !s.starts_with('.') && !s.starts_with('/') {
        s.insert_str(0, "./");
    }
    if s.ends_with(".ts") || s.ends_with(".js") {
        // Strip trailing .ts/.js for the import specifier so the bundler /
        // tsconfig resolution picks the right file.
        s.truncate(s.len() - 3);
    }
    Some(s)
}

/// Compute `target` relative to `base`. Mirrors `pathdiff::diff_paths` —
/// inlined here to avoid pulling a new dependency for one function.
fn diff_paths(target: &std::path::Path, base: &std::path::Path) -> Option<std::path::PathBuf> {
    use std::path::{Component, PathBuf};
    if target.is_absolute() != base.is_absolute() {
        if target.is_absolute() {
            return Some(target.to_path_buf());
        }
        return None;
    }
    let mut ita = target.components();
    let mut itb = base.components();
    let mut comps = vec![];
    loop {
        match (ita.next(), itb.next()) {
            (None, None) => break,
            (Some(a), None) => {
                comps.push(a);
                comps.extend(ita.by_ref());
                break;
            }
            (None, _) => comps.push(Component::ParentDir),
            (Some(a), Some(b)) if comps.is_empty() && a == b => (),
            (Some(_), Some(Component::CurDir)) => (),
            (Some(_), Some(_)) => {
                comps.push(Component::ParentDir);
                for _ in itb {
                    comps.push(Component::ParentDir);
                }
                comps.push(ita.next()?);
                comps.extend(ita.by_ref());
                break;
            }
        }
    }
    Some(comps.iter().map(|c| c.as_os_str()).collect::<PathBuf>())
}

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
}
