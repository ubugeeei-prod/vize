//! Quick fixes from plugin lint rules (#13074).
//!
//! Plugin diagnostics carry SFC-absolute spans, and their fix replaces the
//! reported record's authored span. Fixes of rules declared `fixable: "safe"`
//! are preferred; other fixes are offered but never preferred.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "tower-lsp action titles and edits use std String/HashMap"
)]

use super::{CodeActionService, IdeContext, ranges_overlap, template_position};
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Diagnostic, Range, TextEdit, WorkspaceEdit,
};

impl CodeActionService {
    pub(super) fn collect_plugin_lint_fixes(
        ctx: &IdeContext,
        range: Range,
        requested: &[Diagnostic],
        #[cfg(feature = "native")] stamp: Option<super::versioned_lint_edits::LintEditStamp>,
    ) -> Vec<CodeActionOrCommand> {
        if !ctx.uri.path().ends_with(".vue") {
            return Vec::new();
        }
        let mut actions = Vec::new();
        for (diagnostic, safe) in ctx.state.plugin_lint_fixes(ctx.uri, &ctx.content) {
            let Some(fix) = &diagnostic.fix else {
                continue;
            };
            let diagnostic_range = Range {
                start: template_position(&ctx.content, 0, diagnostic.start as usize),
                end: template_position(&ctx.content, 0, diagnostic.end as usize),
            };
            if !ranges_overlap(&diagnostic_range, &range) {
                continue;
            }
            let Some(diagnostics) = super::template_lint::selected_diagnostics(
                requested,
                diagnostic.rule_name,
                diagnostic_range,
            ) else {
                continue;
            };
            #[cfg(feature = "native")]
            let edit = match stamp {
                Some(stamp) => {
                    let Some(edit) = super::versioned_lint_edits::workspace_edit(
                        ctx,
                        stamp,
                        &ctx.content,
                        0,
                        &fix.edits,
                    ) else {
                        continue;
                    };
                    edit
                }
                None => unversioned_edit(ctx, &fix.edits),
            };
            #[cfg(not(feature = "native"))]
            let edit = unversioned_edit(ctx, &fix.edits);
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: format!("Fix: {}", fix.message),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: (!diagnostics.is_empty()).then_some(diagnostics),
                edit: Some(edit),
                command: None,
                is_preferred: Some(safe),
                disabled: None,
                data: None,
            }));
        }
        actions
    }
}

fn unversioned_edit(ctx: &IdeContext, edits: &[vize_patina::TextEdit]) -> WorkspaceEdit {
    let edits = edits
        .iter()
        .map(|edit| TextEdit {
            range: Range {
                start: template_position(&ctx.content, 0, edit.start as usize),
                end: template_position(&ctx.content, 0, edit.end as usize),
            },
            new_text: edit.new_text.to_string(),
        })
        .collect();
    let mut changes = std::collections::HashMap::new();
    changes.insert(ctx.uri.clone(), edits);
    WorkspaceEdit {
        changes: Some(changes),
        document_changes: None,
        change_annotations: None,
    }
}
