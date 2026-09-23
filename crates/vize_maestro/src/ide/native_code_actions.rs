//! Atomic, versioned TypeScript fixes for exact authored Vue ranges.
#![expect(
    clippy::disallowed_types,
    reason = "tower-lsp lsp_types take std String/HashMap values, built with to_string/format!"
)]

use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, DocumentChanges, OneOf,
    OptionalVersionedTextDocumentIdentifier, Position, Range, TextDocumentEdit, TextEdit,
    WorkspaceEdit,
};
use vize_canon::{LspPosition, LspRange};

use super::code_action::CodeActionService;
use crate::ide::{IdeContext, corsa_support, position_to_offset};

mod binding_origins;
#[cfg(test)]
mod tests;

impl CodeActionService {
    pub(crate) async fn native_actions(
        ctx: &IdeContext<'_>,
        range: Range,
    ) -> Vec<CodeActionOrCommand> {
        if !ctx.state.is_lsp_typecheck_enabled()
            || !ctx.uri.path().ends_with(".vue")
            || ctx.uri.path().ends_with(".art.vue")
        {
            return vec![];
        }
        let Some(version) = ctx
            .state
            .documents
            .get(ctx.uri)
            .map(|document| document.version)
        else {
            return vec![];
        };
        let Some(bridge) = ctx.state.get_corsa_bridge().await else {
            return vec![];
        };
        let Some(document) = corsa_support::open_canonical_virtual_document(ctx, &bridge).await
        else {
            return vec![];
        };
        let Some(virtual_range) = request_range(ctx, &document, range) else {
            return vec![];
        };
        let response = match bridge
            .code_actions(&document.request_uri, virtual_range)
            .await
        {
            Ok(Some(response)) => response,
            Ok(None) => return vec![],
            Err(error) => {
                tracing::warn!(%error, uri = %ctx.uri, "native code-action request failed");
                return vec![];
            }
        };
        let actions = match serde_json::from_value::<Vec<CodeActionOrCommand>>(response) {
            Ok(actions) => actions,
            Err(error) => {
                tracing::warn!(%error, uri = %ctx.uri, "invalid native code-action response");
                return vec![];
            }
        };
        actions
            .into_iter()
            .filter_map(|action| {
                let CodeActionOrCommand::CodeAction(action) = action else {
                    return None;
                };
                map_action(ctx, &document, version, action).map(CodeActionOrCommand::CodeAction)
            })
            .collect()
    }
}

fn request_range(
    ctx: &IdeContext<'_>,
    document: &corsa_support::CanonicalVirtualDocument,
    range: Range,
) -> Option<LspRange> {
    let map = |position: Position| {
        let offset = position_to_offset(&ctx.content, position.line, position.character)?;
        let (line, character) =
            corsa_support::canonical_source_offset_to_position(document, offset)?;
        Some(Position::new(line, character))
    };
    let generated = Range::new(map(range.start)?, map(range.end)?);
    // Navigation mappings may approximate a desugared token. Writable actions
    // require an exact round trip, even when invoked at an empty cursor range.
    if corsa_support::map_canonical_exact_edit_range(ctx, document, generated)? != range {
        return None;
    }
    Some(LspRange {
        start: LspPosition {
            line: generated.start.line,
            character: generated.start.character,
        },
        end: LspPosition {
            line: generated.end.line,
            character: generated.end.character,
        },
    })
}

fn map_action(
    ctx: &IdeContext<'_>,
    document: &corsa_support::CanonicalVirtualDocument,
    version: i32,
    mut action: CodeAction,
) -> Option<CodeAction> {
    if action.command.is_some() || action.kind.as_ref()? != &CodeActionKind::QUICKFIX {
        return None;
    }
    let original = action.edit.take()?;
    if original.changes.is_some() && original.document_changes.is_some() {
        return None;
    }
    let mut edits = Vec::new();
    let mut generated_edits = Vec::new();
    let mut add = |uri: &str, edit: TextEdit| -> Option<()> {
        if uri != document.request_uri.as_str() {
            return None;
        }
        let range = corsa_support::map_canonical_exact_edit_range(ctx, document, edit.range)?;
        generated_edits.push(edit.clone());
        edits.push(TextEdit {
            range,
            new_text: edit.new_text,
        });
        Some(())
    };
    if let Some(changes) = original.changes {
        for (uri, changes) in changes {
            for edit in changes {
                add(uri.as_str(), edit)?;
            }
        }
    }
    if let Some(changes) = original.document_changes {
        let DocumentChanges::Edits(changes) = changes else {
            return None;
        };
        for change in changes {
            for edit in change.edits {
                // Annotations can require confirmation and must not be silently
                // stripped when translating an edit to another document.
                let OneOf::Left(edit) = edit else {
                    return None;
                };
                add(change.text_document.uri.as_str(), edit)?;
            }
        }
    }
    let edits = ordered_edits(edits)?;
    if edits.is_empty() {
        return None;
    }
    binding_origins::preserves_binding_origins(ctx, document, generated_edits)?;
    action.edit = Some(WorkspaceEdit {
        document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
            text_document: OptionalVersionedTextDocumentIdentifier {
                uri: ctx.uri.clone(),
                version: Some(version),
            },
            edits: edits.into_iter().map(OneOf::Left).collect(),
        }])),
        ..Default::default()
    });
    action.diagnostics = None;
    action.data = None;
    Some(action)
}

fn ordered_edits(mut edits: Vec<TextEdit>) -> Option<Vec<TextEdit>> {
    edits.sort_by_key(|edit| (edit.range.start, edit.range.end));
    let mut ordered: Vec<TextEdit> = Vec::new();
    for edit in edits {
        if let Some(previous) = ordered.last_mut() {
            if previous.range == edit.range && edit.range.start == edit.range.end {
                // LSP preserves the original array order for same-position
                // insertions. Coalesce them so every client applies it alike.
                previous.new_text.push_str(&edit.new_text);
                continue;
            }
            if previous.range.end > edit.range.start || previous.range.start == edit.range.start {
                return None;
            }
        }
        ordered.push(edit);
    }
    Some(ordered)
}
