//! Script lint suppression actions over the configured SFC lint pass.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "tower-lsp action titles and edits use std String/HashMap"
)]

use super::{get_line_indent, ranges_overlap, CodeActionService, IdeContext};
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Diagnostic, Position, Range, TextEdit,
    WorkspaceEdit,
};
use vize_atelier_sfc::SfcScriptBlock;
use vize_l0::line_index::offset_to_line_col;

/// One configured full-SFC lint pass used for script suppression actions.
pub(super) struct ScriptLint {
    result: vize_patina::LintResult,
}

impl CodeActionService {
    pub(super) fn lint_script_once(
        ctx: &IdeContext,
        requested: &[Diagnostic],
    ) -> Option<ScriptLint> {
        let descriptor = ctx.descriptor()?;
        let requests_script_action = if requested.is_empty() {
            script_block_at(descriptor, ctx.offset).is_some()
        } else {
            requested.iter().any(|diagnostic| {
                diagnostic.source.as_deref() == Some(crate::ide::diagnostics::sources::LINTER)
                    && crate::ide::position_to_offset(
                        &ctx.content,
                        diagnostic.range.start.line,
                        diagnostic.range.start.character,
                    )
                    .is_some_and(|offset| script_block_at(descriptor, offset).is_some())
            })
        };
        if !requests_script_action {
            return None;
        }
        let linter = crate::ide::diagnostics::patina::linter_for_uri(
            ctx.state,
            ctx.uri,
            ctx.state.lsp_features().ecosystem,
        )?;
        Some(ScriptLint {
            result: linter.lint_sfc(&ctx.content, ctx.uri.path()),
        })
    }

    /// Offer the suppression syntax Patina accepts inside script blocks.
    pub(super) fn collect_script_suppress(
        ctx: &IdeContext,
        range: Range,
        lint: &ScriptLint,
        requested: &[Diagnostic],
    ) -> Vec<CodeActionOrCommand> {
        let Some(descriptor) = ctx.descriptor() else {
            return Vec::new();
        };
        let newline = if ctx.content.contains("\r\n") {
            "\r\n"
        } else {
            "\n"
        };
        let mut actions = Vec::new();

        for lint_diag in &lint.result.diagnostics {
            let start = lint_diag.start as usize;
            let Some(block) = script_block_at(descriptor, start) else {
                continue;
            };
            let diag_range = absolute_range(&ctx.content, start, lint_diag.end as usize);
            if !ranges_overlap(&diag_range, &range) {
                continue;
            }
            let Some(diagnostics) = super::template_lint::selected_diagnostics(
                requested,
                lint_diag.rule_name,
                diag_range,
            ) else {
                continue;
            };

            let relative_start = start.saturating_sub(block.loc.start);
            let edit = script_suppression_edit(
                &ctx.content,
                block.content,
                block.loc.start,
                relative_start,
                lint_diag.rule_name,
                newline,
            );
            let mut changes = std::collections::HashMap::new();
            changes.insert(ctx.uri.clone(), vec![edit]);
            let action = CodeAction {
                title: format!(
                    "Suppress with eslint-disable-next-line ({})",
                    lint_diag.rule_name
                ),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: (!diagnostics.is_empty()).then_some(diagnostics),
                edit: Some(WorkspaceEdit {
                    changes: Some(changes),
                    document_changes: None,
                    change_annotations: None,
                }),
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

fn script_suppression_edit(
    source: &str,
    block_content: &str,
    block_start: usize,
    relative_start: usize,
    rule_name: &str,
    newline: &str,
) -> TextEdit {
    let line_start = block_content
        .get(..relative_start)
        .and_then(|head| head.rfind('\n'))
        .map_or(0, |offset| offset + 1);

    if line_start > 0 {
        let previous_end_with_newline = line_start - 1;
        let previous_end = previous_end_with_newline
            - usize::from(
                block_content
                    .as_bytes()
                    .get(previous_end_with_newline.saturating_sub(1))
                    == Some(&b'\r'),
            );
        let previous_start = block_content
            .get(..previous_end)
            .and_then(|head| head.rfind('\n'))
            .map_or(0, |offset| offset + 1);
        if let Some(previous_line) = block_content.get(previous_start..previous_end) {
            const DIRECTIVE: &str = "// eslint-disable-next-line ";
            if previous_line.trim_start().starts_with(DIRECTIVE) {
                let start = block_start + previous_start;
                let end = block_start + previous_end;
                let (start_line, start_character) = offset_to_line_col(source, start);
                let (end_line, end_character) = offset_to_line_col(source, end);
                return TextEdit {
                    range: Range::new(
                        Position::new(start_line, start_character),
                        Position::new(end_line, end_character),
                    ),
                    new_text: format!("{previous_line}, {rule_name}"),
                };
            }
        }
    }

    let insert_offset = block_start + line_start;
    let (line, character) = offset_to_line_col(source, insert_offset);
    let insert_position = Position { line, character };
    let indent = get_line_indent(block_content, relative_start);
    TextEdit {
        range: Range::new(insert_position, insert_position),
        new_text: format!("{indent}// eslint-disable-next-line {rule_name}{newline}"),
    }
}

fn script_block_at<'a>(
    descriptor: &'a vize_atelier_sfc::SfcDescriptor<'a>,
    offset: usize,
) -> Option<&'a SfcScriptBlock<'a>> {
    descriptor
        .script_setup
        .as_ref()
        .filter(|block| offset >= block.loc.start && offset < block.loc.end)
        .or_else(|| {
            descriptor
                .script
                .as_ref()
                .filter(|block| offset >= block.loc.start && offset < block.loc.end)
        })
}

fn absolute_range(source: &str, start: usize, end: usize) -> Range {
    let (start_line, start_character) = offset_to_line_col(source, start);
    let (end_line, end_character) = offset_to_line_col(source, end);
    Range::new(
        Position::new(start_line, start_character),
        Position::new(end_line, end_character),
    )
}
