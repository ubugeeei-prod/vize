//! Template lint actions over one resident-descriptor lint pass.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "tower-lsp action titles and edits use std String/HashMap"
)]

use super::{CodeActionService, IdeContext, get_line_indent, ranges_overlap, template_position};
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Range, TextEdit, WorkspaceEdit,
};

/// One template lint pass over the resident descriptor, shared across the
/// lint-based collectors in a single request.
pub(super) struct TemplateLint {
    /// Template block content (the text the linter ran against).
    content: String,
    /// Byte offset where the template content starts inside the SFC.
    start_offset: usize,
    /// Lint diagnostics for the template block.
    result: vize_patina::LintResult,
}

impl CodeActionService {
    /// Read the resident SFC descriptor and run the template linter once.
    /// Share its content, source offset and result between lint-fix and
    /// `@vize:forget` collectors within the same request.
    pub(super) fn lint_template_once(ctx: &IdeContext) -> Option<TemplateLint> {
        let descriptor = ctx.descriptor()?;
        let template = descriptor.template.as_ref()?;
        let linter = vize_patina::Linter::new();
        let result = linter.lint_template(&template.content, ctx.uri.path());
        Some(TemplateLint {
            content: template.content.to_string(),
            start_offset: template.loc.start,
            result,
        })
    }

    /// Collect lint fix actions from vize_patina diagnostics.
    pub(super) fn collect_lint_fixes(
        ctx: &IdeContext,
        range: Range,
        lint: &TemplateLint,
    ) -> Vec<CodeActionOrCommand> {
        let mut actions = Vec::new();

        for lint_diag in &lint.result.diagnostics {
            // Check if diagnostic has a fix
            let Some(ref fix) = lint_diag.fix else {
                continue;
            };

            // Convert lint diagnostic position to SFC position
            let diag_range = Range {
                start: template_position(&ctx.content, lint.start_offset, lint_diag.start as usize),
                end: template_position(&ctx.content, lint.start_offset, lint_diag.end as usize),
            };

            // Check if the diagnostic range overlaps with the requested range
            if !ranges_overlap(&diag_range, &range) {
                continue;
            }

            // Convert fix edits to LSP TextEdits
            let edits: Vec<TextEdit> = fix
                .edits
                .iter()
                .map(|edit| TextEdit {
                    range: Range {
                        start: template_position(
                            &ctx.content,
                            lint.start_offset,
                            edit.start as usize,
                        ),
                        end: template_position(&ctx.content, lint.start_offset, edit.end as usize),
                    },
                    new_text: edit.new_text.to_string(),
                })
                .collect();

            // Create workspace edit
            let mut changes = std::collections::HashMap::new();
            changes.insert(ctx.uri.clone(), edits);

            let workspace_edit = WorkspaceEdit {
                changes: Some(changes),
                document_changes: None,
                change_annotations: None,
            };

            // Create code action
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
    pub(super) fn collect_forget_suppress(
        ctx: &IdeContext,
        range: Range,
        lint: &TemplateLint,
    ) -> Vec<CodeActionOrCommand> {
        let mut actions = Vec::new();

        let template_content = lint.content.as_str();
        let newline = if ctx
            .content
            .split_once('\n')
            .is_some_and(|(first_line, _)| first_line.ends_with('\r'))
        {
            "\r\n"
        } else {
            "\n"
        };

        for lint_diag in &lint.result.diagnostics {
            // Convert diagnostic position to SFC position
            let diag_range = Range {
                start: template_position(&ctx.content, lint.start_offset, lint_diag.start as usize),
                end: template_position(&ctx.content, lint.start_offset, lint_diag.end as usize),
            };

            if !ranges_overlap(&diag_range, &range) {
                continue;
            }

            // Compute indentation of the diagnostic line
            let indent = get_line_indent(template_content, lint_diag.start as usize);

            // The first content line can follow `<template>` on the same line.
            let line_start = template_content
                .get(..lint_diag.start as usize)
                .and_then(|head| head.rfind('\n'))
                .map_or(0, |offset| offset + 1);
            let insert_pos = template_position(&ctx.content, lint.start_offset, line_start);

            let new_text = format!(
                "{}<!-- @vize:forget {} -->{newline}",
                indent, lint_diag.rule_name
            );

            let edit = TextEdit {
                range: Range {
                    start: insert_pos,
                    end: insert_pos,
                },
                new_text,
            };

            let mut changes = std::collections::HashMap::new();
            changes.insert(ctx.uri.clone(), vec![edit]);

            let workspace_edit = WorkspaceEdit {
                changes: Some(changes),
                document_changes: None,
                change_annotations: None,
            };

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
}
