use std::collections::HashMap;

use tower_lsp::lsp_types::{
    DocumentChangeOperation, DocumentChanges, OneOf, PrepareRenameResponse, Range, TextEdit, Url,
    WorkspaceEdit,
};
use vize_canon::CorsaBridge;
use vize_s0::{FxHashSet, String, cstr};

use crate::ide::{IdeContext, ReferencesService, corsa_support};

mod event_rename;
mod execute;
mod failure;

pub(super) use execute::rename;
#[cfg(test)]
pub(super) use execute::{CanonicalRenameStage, rename_strict, rename_strict_traced};
use failure::CanonicalFailure;

pub(super) enum Answer<T> {
    Unavailable,
    Available(Option<T>),
}

pub(super) async fn prepare(
    ctx: &IdeContext<'_>,
    bridge: Option<&CorsaBridge>,
) -> Answer<PrepareRenameResponse> {
    match prepare_strict(ctx, bridge).await {
        Ok(answer) => answer,
        Err(error) => error.into_lenient_answer(),
    }
}

pub(super) async fn prepare_strict(
    ctx: &IdeContext<'_>,
    bridge: Option<&CorsaBridge>,
) -> Result<Answer<PrepareRenameResponse>, CanonicalFailure> {
    let Some(bridge) = initialized_bridge(bridge) else {
        return Ok(Answer::Unavailable);
    };
    let Some(document) = corsa_support::open_canonical_virtual_document_strict(ctx, bridge)
        .await
        .map_err(CanonicalFailure::FallbackBridge)?
    else {
        return Ok(Answer::Unavailable);
    };
    let Some((line, character)) = event_rename::semantic_position(ctx, &document)
        .or_else(|| corsa_support::canonical_source_offset_to_position(&document, ctx.offset))
    else {
        return Ok(Answer::Unavailable);
    };
    let response = bridge
        .prepare_rename(&document.request_uri, line, character)
        .await
        .map_err(CanonicalFailure::FallbackBridge)?;
    let response = response
        .map(|response| {
            serde_json::from_value(response).map_err(|error| CanonicalFailure::InvalidResponse {
                operation: "prepareRename",
                message: cstr!("{error}"),
            })
        })
        .transpose()?;
    Ok(Answer::Available(response.and_then(|response| {
        event_rename::prepare_range(ctx)
            .map(PrepareRenameResponse::Range)
            .or_else(|| corsa_support::map_canonical_prepare_rename(ctx, &document, response))
    })))
}

fn style_workspace_edit(
    query: &IdeContext<'_>,
    semantic: &[WorkspaceEdit],
    new_name: &str,
) -> Option<WorkspaceEdit> {
    let mut seeds = FxHashSet::default();
    let mut changes = HashMap::new();
    collect_style_edits(
        query,
        query.uri,
        query.offset,
        new_name,
        &mut seeds,
        &mut changes,
    );
    for edit in semantic {
        for (uri, range) in authored_edit_ranges(edit) {
            let Some(source) = query.state.documents.text(&uri) else {
                continue;
            };
            let Some(offset) =
                crate::ide::position_to_offset(&source, range.start.line, range.start.character)
            else {
                continue;
            };
            collect_style_edits(query, &uri, offset, new_name, &mut seeds, &mut changes);
        }
    }
    (!changes.is_empty()).then_some(WorkspaceEdit {
        changes: Some(changes),
        document_changes: None,
        change_annotations: None,
    })
}

fn collect_style_edits(
    query: &IdeContext<'_>,
    uri: &Url,
    offset: usize,
    new_name: &str,
    seeds: &mut FxHashSet<(Url, String)>,
    changes: &mut HashMap<Url, Vec<TextEdit>>,
) {
    let Some(source) = query.state.documents.text(uri) else {
        return;
    };
    let Some(word) = crate::ide::token_at_offset(&source, offset, |byte| {
        byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$')
    }) else {
        return;
    };
    if !seeds.insert((uri.clone(), word.clone().into())) {
        return;
    }
    let Some(ctx) = IdeContext::new(query.state, uri, offset) else {
        return;
    };
    for location in ReferencesService::find_references_in_style(&ctx, &word) {
        let edits = changes.entry(location.uri).or_default();
        if !edits.iter().any(|edit| edit.range == location.range) {
            edits.push(TextEdit {
                range: location.range,
                new_text: new_name.to_string(),
            });
        }
    }
}

fn authored_edit_ranges(edit: &WorkspaceEdit) -> Vec<(Url, Range)> {
    let mut ranges = Vec::new();
    if let Some(changes) = &edit.changes {
        for (uri, edits) in changes {
            ranges.extend(edits.iter().map(|edit| (uri.clone(), edit.range)));
        }
    }
    if let Some(changes) = &edit.document_changes {
        match changes {
            DocumentChanges::Edits(edits) => {
                for edit in edits {
                    ranges.extend(
                        edit.edits.iter().map(|entry| {
                            (edit.text_document.uri.clone(), annotatable_range(entry))
                        }),
                    );
                }
            }
            DocumentChanges::Operations(operations) => {
                for operation in operations {
                    if let DocumentChangeOperation::Edit(edit) = operation {
                        ranges.extend(edit.edits.iter().map(|entry| {
                            (edit.text_document.uri.clone(), annotatable_range(entry))
                        }));
                    }
                }
            }
        }
    }
    ranges
}

fn linked_positions(
    document: &corsa_support::CanonicalVirtualDocument,
    edit: &WorkspaceEdit,
) -> Vec<corsa_support::CanonicalSemanticPosition> {
    let mut positions = FxHashSet::default();
    let mut push = |uri: &Url, range: Range| {
        if let Some(position) = corsa_support::linked_semantic_position(
            document,
            uri.as_str(),
            &corsa_support::tower_range(range),
        ) {
            positions.insert(position);
        }
    };
    if let Some(changes) = &edit.changes {
        for (uri, edits) in changes {
            for edit in edits {
                push(uri, edit.range);
            }
        }
    }
    if let Some(changes) = &edit.document_changes {
        match changes {
            DocumentChanges::Edits(edits) => {
                for edit in edits {
                    for entry in &edit.edits {
                        push(&edit.text_document.uri, annotatable_range(entry));
                    }
                }
            }
            DocumentChanges::Operations(operations) => {
                for operation in operations {
                    if let DocumentChangeOperation::Edit(edit) = operation {
                        for entry in &edit.edits {
                            push(&edit.text_document.uri, annotatable_range(entry));
                        }
                    }
                }
            }
        }
    }
    positions.into_iter().collect()
}

fn annotatable_range(
    edit: &OneOf<tower_lsp::lsp_types::TextEdit, tower_lsp::lsp_types::AnnotatedTextEdit>,
) -> Range {
    match edit {
        OneOf::Left(edit) => edit.range,
        OneOf::Right(edit) => edit.text_edit.range,
    }
}

fn initialized_bridge(bridge: Option<&CorsaBridge>) -> Option<&CorsaBridge> {
    bridge.filter(|bridge| bridge.is_initialized())
}
