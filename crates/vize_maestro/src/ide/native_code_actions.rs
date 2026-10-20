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
use import_projection::ImportProjection;

mod binding_origins;
mod import_projection;
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

/// Whole-document TypeScript source actions this server forwards to tsgo and
/// advertises. Each has a real-tsgo whole-SFC oracle.
pub(crate) const NATIVE_SOURCE_KINDS: [&str; 4] = [
    "source.organizeImports",
    "source.sortImports",
    "source.removeUnusedImports",
    FIX_ALL,
];

const FIX_ALL: &str = "source.fixAll";

impl CodeActionService {
    /// Organize, sort or remove imports, or fix all, in a Vue SFC script.
    pub(crate) async fn native_source_actions(
        ctx: &IdeContext<'_>,
        kinds: &[CodeActionKind],
    ) -> Vec<CodeActionOrCommand> {
        if kinds.is_empty()
            || !ctx.state.is_lsp_typecheck_enabled()
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
        let (fix_all, imports): (Vec<_>, Vec<_>) = kinds
            .iter()
            .cloned()
            .partition(|kind| kind.as_str() == FIX_ALL);
        let mut actions = Self::import_source_actions(ctx, &imports, version).await;
        if !fix_all.is_empty() {
            actions.extend(Self::fix_all_actions(ctx, version).await);
        }
        actions
    }

    /// Fix all runs on the canonical document, where the checker sees the
    /// whole SFC. Like a quick fix, every edit must map exactly onto authored
    /// text and keep its references' bindings, or the action is dropped whole.
    async fn fix_all_actions(ctx: &IdeContext<'_>, version: i32) -> Vec<CodeActionOrCommand> {
        let Some(bridge) = ctx.state.get_corsa_bridge().await else {
            return vec![];
        };
        let Some(document) = corsa_support::open_canonical_virtual_document(ctx, &bridge).await
        else {
            return vec![];
        };
        let response = match bridge
            .source_actions(&document.request_uri, vec![FIX_ALL.into()])
            .await
        {
            Ok(Some(response)) => response,
            Ok(None) => return vec![],
            Err(error) => {
                tracing::warn!(%error, uri = %ctx.uri, "native fix-all request failed");
                return vec![];
            }
        };
        let actions = match serde_json::from_value::<Vec<CodeActionOrCommand>>(response) {
            Ok(actions) => actions,
            Err(error) => {
                tracing::warn!(%error, uri = %ctx.uri, "invalid native fix-all response");
                return vec![];
            }
        };
        actions
            .into_iter()
            .filter_map(|action| {
                let CodeActionOrCommand::CodeAction(action) = action else {
                    return None;
                };
                if action.kind.as_ref()?.as_str() != FIX_ALL {
                    return None;
                }
                map_action_edits(ctx, &document, version, action)
                    .map(CodeActionOrCommand::CodeAction)
            })
            .collect()
    }

    /// tsgo answers import actions on [`ImportProjection`]; every edit must
    /// land on the authored script or the action is dropped whole.
    async fn import_source_actions(
        ctx: &IdeContext<'_>,
        kinds: &[CodeActionKind],
        version: i32,
    ) -> Vec<CodeActionOrCommand> {
        if kinds.is_empty() {
            return vec![];
        }
        let Some(projection) = ImportProjection::build(ctx) else {
            return vec![];
        };
        let Some(bridge) = ctx.state.get_corsa_bridge().await else {
            return vec![];
        };
        let request_uri = match bridge
            .open_or_update_virtual_document(&projection.path, &projection.text)
            .await
        {
            Ok(uri) => uri,
            Err(error) => {
                tracing::warn!(%error, uri = %ctx.uri, "import projection could not be opened");
                return vec![];
            }
        };
        let requested = kinds.iter().map(|kind| kind.as_str().into()).collect();
        let response = bridge.source_actions(&request_uri, requested).await;
        if let Err(error) = bridge.close_virtual_document(&request_uri).await {
            tracing::warn!(%error, uri = %ctx.uri, "import projection could not be closed");
        }
        let response = match response {
            Ok(Some(response)) => response,
            Ok(None) => return vec![],
            Err(error) => {
                tracing::warn!(%error, uri = %ctx.uri, "native source-action request failed");
                return vec![];
            }
        };
        let actions = match serde_json::from_value::<Vec<CodeActionOrCommand>>(response) {
            Ok(actions) => actions,
            Err(error) => {
                tracing::warn!(%error, uri = %ctx.uri, "invalid native source-action response");
                return vec![];
            }
        };
        actions
            .into_iter()
            .filter_map(|action| {
                let CodeActionOrCommand::CodeAction(action) = action else {
                    return None;
                };
                if !action
                    .kind
                    .as_ref()
                    .is_some_and(|kind| kinds.contains(kind))
                {
                    return None;
                }
                map_projection_action(ctx, &projection, &request_uri, version, action)
                    .map(CodeActionOrCommand::CodeAction)
            })
            .collect()
    }
}

fn map_projection_action(
    ctx: &IdeContext<'_>,
    projection: &ImportProjection,
    request_uri: &str,
    version: i32,
    mut action: CodeAction,
) -> Option<CodeAction> {
    if action.command.is_some() {
        return None;
    }
    let original = action.edit.take()?;
    let mut edits = Vec::new();
    let mut add = |uri: &str, edit: TextEdit| -> Option<()> {
        if uri != request_uri {
            return None;
        }
        edits.push(TextEdit {
            range: projection.authored_range(ctx, edit.range)?,
            new_text: edit.new_text,
        });
        Some(())
    };
    if original.changes.is_some() && original.document_changes.is_some() {
        return None;
    }
    for (uri, changes) in original.changes.into_iter().flatten() {
        for edit in changes {
            add(uri.as_str(), edit)?;
        }
    }
    if let Some(changes) = original.document_changes {
        let DocumentChanges::Edits(changes) = changes else {
            return None;
        };
        for change in changes {
            for edit in change.edits {
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
    action: CodeAction,
) -> Option<CodeAction> {
    if action.kind.as_ref()? != &CodeActionKind::QUICKFIX {
        return None;
    }
    map_action_edits(ctx, document, version, action)
}

/// Translate one backend action's edits onto the authored SFC, all or none.
fn map_action_edits(
    ctx: &IdeContext<'_>,
    document: &corsa_support::CanonicalVirtualDocument,
    version: i32,
    mut action: CodeAction,
) -> Option<CodeAction> {
    if action.command.is_some() {
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
        let new_text = relayout(
            &edit.new_text,
            line_layout(&document.virtual_result.code, edit.range.start.line).0,
            line_layout(&ctx.content, range.start.line),
        );
        generated_edits.push(edit);
        edits.push(TextEdit { range, new_text });
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

/// The leading whitespace of a 0-based line, and whether it ends in CRLF.
fn line_layout(text: &str, line: u32) -> (&str, bool) {
    let line = text.split('\n').nth(line as usize).unwrap_or_default();
    let indent = line.len() - line.trim_start_matches([' ', '\t']).len();
    (line.get(..indent).unwrap_or_default(), line.ends_with('\r'))
}

/// tsgo lays inserted lines out for the generated line, which can be nested
/// deeper than the authored one (`<script setup>` sits inside `__setup()`)
/// and ends in LF. Use the authored line's base indent and line ending,
/// unless an inserted line does not start with the generated indent.
fn relayout(text: &str, generated: &str, (indent, crlf): (&str, bool)) -> String {
    let mut lines = text.split('\n');
    let first = lines.next().unwrap_or_default();
    let rest: Vec<_> = lines.collect();
    if rest
        .iter()
        .any(|line| !line.trim().is_empty() && !line.starts_with(generated))
    {
        return text.to_string();
    }
    let mut out = String::from(first);
    for line in rest {
        if crlf && !out.ends_with('\r') {
            out.push('\r');
        }
        out.push('\n');
        match line.strip_prefix(generated) {
            Some(tail) => {
                out.push_str(indent);
                out.push_str(tail);
            }
            None => out.push_str(line),
        }
    }
    out
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
