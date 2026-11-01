//! Code action provider for Vue SFC files.
//!
//! Provides code actions for:
//! - Lint fixes from vize_patina
//! - Quick fixes for common issues
//! - Refactoring actions
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "tower-lsp code actions take std String titles/edits and a std HashMap of changes"
)]

use super::IdeContext;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Diagnostic, Position, Range, TextEdit,
    WorkspaceEdit,
};
// Shared, UTF-16-correct offset->(line, column) conversion (#1389).
use vize_l0::line_index::offset_to_line_col;

#[cfg(all(test, unix))]
mod auto_import_literal_tests;
#[cfg(test)]
mod diagnostic_selection_tests;
mod ref_value;
#[cfg(test)]
mod ref_value_tests;
mod script_bindings;
mod script_lint;
#[cfg(test)]
mod script_lint_tests;
#[cfg(test)]
mod source_position_tests;
mod template_lint;
#[cfg(feature = "native")]
mod versioned_lint_edits;

use script_bindings::sfc_script_declares;

/// Code action service for providing quick fixes and refactorings.
pub struct CodeActionService;

impl CodeActionService {
    /// Get code actions for the given context and range.
    pub fn code_actions(ctx: &IdeContext, range: Range) -> Vec<CodeActionOrCommand> {
        Self::code_actions_for_diagnostics(ctx, range, &[])
    }

    /// Select lint fixes by current rule identity, retaining cursor-only actions.
    pub(crate) fn code_actions_for_diagnostics(
        ctx: &IdeContext,
        range: Range,
        diagnostics: &[Diagnostic],
    ) -> Vec<CodeActionOrCommand> {
        Self::collect_actions(
            ctx,
            range,
            diagnostics,
            #[cfg(feature = "native")]
            None,
        )
    }

    fn collect_actions(
        ctx: &IdeContext,
        range: Range,
        diagnostics: &[Diagnostic],
        #[cfg(feature = "native")] stamp: Option<versioned_lint_edits::LintEditStamp>,
    ) -> Vec<CodeActionOrCommand> {
        let mut actions = Vec::new();

        // Run the template linter once over the resident descriptor; both the
        // lint-fix and `@vize:forget` collectors share that result.
        if let Some(lint) = Self::lint_template_once(ctx) {
            actions.extend(Self::collect_lint_fixes(
                ctx,
                range,
                &lint,
                diagnostics,
                #[cfg(feature = "native")]
                stamp,
            ));
            actions.extend(Self::collect_forget_suppress(
                ctx,
                range,
                &lint,
                diagnostics,
            ));
        }

        if let Some(lint) = Self::lint_script_once(ctx, diagnostics) {
            actions.extend(Self::collect_script_suppress(
                ctx,
                range,
                &lint,
                diagnostics,
            ));
        }

        // Vue-flavored quick fixes: today this surfaces a "Wrap with `.value`"
        // edit when the cursor sits on a known reactive ref. See #691.
        actions.extend(Self::collect_wrap_with_value(ctx));
        actions.extend(Self::collect_unwrap_value(ctx));
        actions.extend(Self::collect_auto_import(ctx, range));

        actions
    }

    /// Offer "Auto-import from <path>" when the cursor sits on an identifier
    /// that the current SFC's script doesn't declare, but the workspace
    /// index built in #722 knows about.
    fn collect_auto_import(ctx: &IdeContext, range: Range) -> Vec<CodeActionOrCommand> {
        let _ = range;
        let Some((_, identifier)) = identifier_at_cursor(&ctx.content, ctx.offset) else {
            return Vec::new();
        };
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
        let Some(descriptor) = ctx.descriptor() else {
            return Vec::new();
        };
        if sfc_script_declares(descriptor, identifier) {
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
        let entry =
            crate::ide::auto_import::AutoImportIndex::find_in_directory(&workspace_dir, identifier);
        let Some(entry) = entry else {
            return Vec::new();
        };

        let import_specifier = match relative_specifier(&workspace_dir, &entry.source) {
            Some(s) => s,
            None => return Vec::new(),
        };
        let mut import_statement = match entry.kind {
            crate::ide::auto_import::AutoImportKind::VueComponent => {
                format!("import {} from ", entry.name)
            }
            crate::ide::auto_import::AutoImportKind::Composable => {
                format!("import {{ {} }} from ", entry.name)
            }
        };
        push_import_specifier(&mut import_statement, &import_specifier);
        import_statement.push('\n');

        // Insert on the first line of script setup if present, otherwise of
        // the regular script block, not after `<script …>` on its own line.
        let Some(script) = descriptor
            .script_setup
            .as_ref()
            .or(descriptor.script.as_ref())
        else {
            return Vec::new();
        };
        let leading_newline = script
            .content
            .strip_prefix("\r\n")
            .or_else(|| script.content.strip_prefix('\n'))
            .map_or(0, |rest| script.content.len() - rest.len());
        let insert_offset = script.loc.start + leading_newline;
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

    /// Get all available fixes for a document (for "fix all" actions).
    pub fn get_all_fixes(ctx: &IdeContext) -> Option<WorkspaceEdit> {
        let descriptor = ctx.descriptor()?;
        let template = descriptor.template.as_ref()?;

        let linter = vize_patina::Linter::new();
        let result = linter.lint_template(&template.content, ctx.uri.path());

        let mut all_edits: Vec<TextEdit> = Vec::new();

        for lint_diag in result.diagnostics {
            if let Some(ref fix) = lint_diag.fix {
                for edit in &fix.edits {
                    all_edits.push(TextEdit {
                        range: Range {
                            start: template_position(
                                &ctx.content,
                                template.loc.start,
                                edit.start as usize,
                            ),
                            end: template_position(
                                &ctx.content,
                                template.loc.start,
                                edit.end as usize,
                            ),
                        },
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

        let mut changes = std::collections::HashMap::new();
        changes.insert(ctx.uri.clone(), filtered_edits);

        Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        })
    }
}

/// Preserve ordinary edit bytes and encode paths that need a JS literal escape.
fn push_import_specifier(output: &mut String, value: &str) {
    if value
        .bytes()
        .any(|byte| matches!(byte, b'\'' | b'\\' | 0..=0x1f))
    {
        let mut literal = vize_l0::String::default();
        vize_l0::js_string::push_js_string_literal(&mut literal, value);
        output.push_str(literal.as_str());
    } else {
        output.push('\'');
        output.push_str(value);
        output.push('\'');
    }
}

fn template_position(source: &str, template_start: usize, offset: usize) -> Position {
    let (line, character) = offset_to_line_col(source, template_start + offset);
    Position { line, character }
}

/// Get the leading whitespace (indentation) for the line containing the given byte offset.
fn get_line_indent(source: &str, offset: usize) -> &str {
    // Find the start of the line
    let line_start = source
        .as_bytes()
        .iter()
        .take(offset)
        .rposition(|&b| b == b'\n')
        .map_or(0, |pos| pos + 1);

    // Collect whitespace from the start of the line
    let rest = source.get(line_start..).unwrap_or_default();
    let indent_len = rest
        .bytes()
        .take_while(|b| *b == b' ' || *b == b'\t')
        .count();

    rest.get(..indent_len).unwrap_or_default()
}

/// Check if two ranges overlap.
/// Return the byte range of the identifier under the cursor, or `None` when
/// the cursor is not on an identifier.
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
    if let Some(suffix) = [".mts", ".cts", ".mjs", ".cjs", ".ts", ".js"]
        .into_iter()
        .find(|suffix| s.ends_with(suffix))
    {
        s.truncate(s.len() - suffix.len());
    }
    Some(s)
}

/// Compute `target` relative to `base`; inlined to avoid a dependency.
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
            (Some(a), Some(_)) => {
                comps.push(Component::ParentDir);
                for _ in itb {
                    comps.push(Component::ParentDir);
                }
                comps.push(a);
                comps.extend(ita.by_ref());
                break;
            }
        }
    }
    Some(comps.iter().map(|c| c.as_os_str()).collect::<PathBuf>())
}

fn identifier_at_cursor(content: &str, offset: usize) -> Option<(std::ops::Range<usize>, &str)> {
    let end = offset.min(content.len());
    let (before, after) = content.as_bytes().split_at_checked(end)?;
    let start = end
        - before
            .iter()
            .rev()
            .take_while(|b| is_ident_byte(**b))
            .count();
    let walk = end + after.iter().take_while(|b| is_ident_byte(**b)).count();
    if start == walk {
        return None;
    }
    Some((start..walk, content.get(start..walk)?))
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

fn ranges_overlap(a: &Range, b: &Range) -> bool {
    // Ranges overlap if neither is completely before or after the other
    !(a.end.line < b.start.line
        || (a.end.line == b.start.line && a.end.character < b.start.character)
        || b.end.line < a.start.line
        || (b.end.line == a.start.line && b.end.character < a.start.character))
}

#[cfg(test)]
mod tests;
