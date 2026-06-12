//! Code actions for `.jsx`/`.tsx` Vue components (#1498).
//!
//! Surfaces the fixable Patina/JSX-compiler diagnostics on a `.jsx`/`.tsx`
//! document as quickfix code actions, the JSX parallel to the SFC
//! [`CodeActionService`](crate::ide::CodeActionService) lint-fix collector.
//!
//! It reuses the **same** plumbing: [`vize_patina::lint_jsx`] runs the shared
//! template lint rules over the lowered JSX (so e.g. a `v-for` missing `:key`
//! yields the identical fixable diagnostic an SFC template would), and each
//! diagnostic carrying a [`Fix`](vize_patina) becomes a
//! [`CodeActionKind::QUICKFIX`] whose edits are the fix edits. Crucially —
//! unlike the SFC template path, whose lint offsets are relative to the
//! `<template>` block — `lint_jsx` reports offsets in the **original source**,
//! so the edits map straight onto `.jsx`/`.tsx` coordinates with no block-offset
//! translation.
//!
//! This is a lint-based (parse-only) provider, like the SFC code-action handler;
//! it needs no Corsa bridge and is not gated on `typeChecker.jsxTypecheck`.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use std::collections::HashMap;

use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Position, Range, TextEdit, Url, WorkspaceEdit,
};
use vize_atelier_jsx::JsxLang;
use vize_carton::line_index::offset_to_line_col;

/// Code-action provider for `.jsx`/`.tsx` components.
pub struct JsxCodeActionService;

impl JsxCodeActionService {
    /// Collect quickfix code actions for the fixable diagnostics overlapping
    /// `range` in a `.jsx`/`.tsx` document.
    pub fn code_actions(content: &str, uri: &Url, range: Range) -> Vec<CodeActionOrCommand> {
        let lang = JsxLang::from_path(uri.path());
        let result = vize_patina::lint_jsx(content, uri.path(), lang);

        let mut actions = Vec::new();
        for diag in &result.diagnostics {
            let Some(ref fix) = diag.fix else {
                continue;
            };

            // The diagnostic's own span (original-source offsets) decides whether
            // this fix is relevant to the requested range.
            let diag_range = offsets_to_range(content, diag.start, diag.end);
            if !ranges_overlap(&diag_range, &range) {
                continue;
            }

            let edits: Vec<TextEdit> = fix
                .edits
                .iter()
                .map(|edit| TextEdit {
                    range: offsets_to_range(content, edit.start, edit.end),
                    new_text: edit.new_text.to_string(),
                })
                .collect();

            let mut changes = HashMap::new();
            changes.insert(uri.clone(), edits);

            #[allow(clippy::disallowed_macros)]
            let action = CodeAction {
                title: format!("Fix: {}", fix.message),
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
            actions.push(CodeActionOrCommand::CodeAction(action));
        }

        actions
    }
}

/// Convert an original-source byte range into an LSP [`Range`] (UTF-16 columns).
fn offsets_to_range(content: &str, start: u32, end: u32) -> Range {
    let (start_line, start_col) = offset_to_line_col(content, start as usize);
    let (end_line, end_col) = offset_to_line_col(content, end as usize);
    Range {
        start: Position {
            line: start_line,
            character: start_col,
        },
        end: Position {
            line: end_line,
            character: end_col,
        },
    }
}

/// Whether two LSP ranges overlap (touching endpoints count, so a cursor at the
/// diagnostic's start still offers the fix).
fn ranges_overlap(a: &Range, b: &Range) -> bool {
    !(a.end.line < b.start.line
        || (a.end.line == b.start.line && a.end.character < b.start.character)
        || b.end.line < a.start.line
        || (b.end.line == a.start.line && b.end.character < a.start.character))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quickfixes(source: &str, range: Range) -> Vec<CodeActionOrCommand> {
        let uri = Url::parse("file:///tmp/Comp.tsx").unwrap();
        JsxCodeActionService::code_actions(source, &uri, range)
    }

    fn whole_doc_range() -> Range {
        Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: u32::MAX,
                character: 0,
            },
        }
    }

    #[test]
    fn surfaces_quickfix_for_fixable_jsx_diagnostic() {
        // Multiple consecutive spaces inside an opening tag is a fixable
        // template-rule diagnostic (`vue/no-multi-spaces`); `lint_jsx` reports it
        // on the lowered JSX, so it must surface as a quickfix.
        let source = "const C = () => <div    class=\"a\">x</div>;\n";
        let actions = quickfixes(source, whole_doc_range());
        let has_quickfix = actions.iter().any(|action| match action {
            CodeActionOrCommand::CodeAction(a) => a.kind == Some(CodeActionKind::QUICKFIX),
            _ => false,
        });
        assert!(
            has_quickfix,
            "expected at least one quickfix code action, got: {actions:?}"
        );
    }

    #[test]
    fn quickfix_carries_a_workspace_edit() {
        let source = "const C = () => <div    class=\"a\">x</div>;\n";
        let actions = quickfixes(source, whole_doc_range());
        let action = actions.iter().find_map(|action| match action {
            CodeActionOrCommand::CodeAction(a) if a.kind == Some(CodeActionKind::QUICKFIX) => {
                Some(a)
            }
            _ => None,
        });
        let action = action.expect("a quickfix action");
        let edit = action.edit.as_ref().expect("quickfix carries an edit");
        let changes = edit.changes.as_ref().expect("edit has changes");
        assert!(changes.values().any(|edits| !edits.is_empty()));
    }

    #[test]
    fn no_actions_for_clean_component() {
        let source = "const C = () => <div class=\"a\">hi</div>;\n";
        let actions = quickfixes(source, whole_doc_range());
        assert!(
            actions.is_empty(),
            "clean component must not offer quickfixes, got: {actions:?}"
        );
    }

    #[test]
    fn no_actions_when_range_misses_diagnostic() {
        let source = "const C = (props: { items: number[] }) => <ul><li v-for={item in props.items}>x</li></ul>;\n";
        // A zero-width range on the very first column, far from the `v-for`.
        let range = Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 1,
            },
        };
        let actions = quickfixes(source, range);
        assert!(
            actions.is_empty(),
            "a range that misses the diagnostic must offer no fix, got: {actions:?}"
        );
    }
}
