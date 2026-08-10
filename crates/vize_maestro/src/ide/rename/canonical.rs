use std::collections::HashMap;

use tower_lsp::lsp_types::{
    DocumentChangeOperation, DocumentChanges, OneOf, PrepareRenameResponse, Range, TextEdit, Url,
    WorkspaceEdit,
};
use vize_canon::CorsaBridge;
use vize_carton::{FxHashSet, String};

use crate::ide::{IdeContext, ReferencesService, corsa_support};

mod event_rename;

pub(super) enum Answer<T> {
    Unavailable,
    Available(Option<T>),
}

pub(super) async fn prepare(
    ctx: &IdeContext<'_>,
    bridge: Option<&CorsaBridge>,
) -> Answer<PrepareRenameResponse> {
    let Some(bridge) = initialized_bridge(bridge) else {
        return Answer::Unavailable;
    };
    let Some(document) = corsa_support::open_canonical_virtual_document(ctx, bridge).await else {
        return Answer::Unavailable;
    };
    let Some((line, character)) = event_rename::semantic_position(ctx, &document)
        .or_else(|| corsa_support::canonical_source_offset_to_position(&document, ctx.offset))
    else {
        return Answer::Unavailable;
    };
    let response = match bridge
        .prepare_rename(&document.request_uri, line, character)
        .await
    {
        Ok(response) => response,
        Err(_) => return Answer::Unavailable,
    };
    let response = response.and_then(|response| serde_json::from_value(response).ok());
    Answer::Available(response.and_then(|response| {
        event_rename::prepare_range(ctx)
            .map(PrepareRenameResponse::Range)
            .or_else(|| corsa_support::map_canonical_prepare_rename(ctx, &document, response))
    }))
}

pub(super) async fn rename(
    ctx: &IdeContext<'_>,
    new_name: &str,
    bridge: Option<&CorsaBridge>,
) -> Answer<WorkspaceEdit> {
    let rename_kind = event_rename::query_kind(ctx);
    let Some(semantic_name) = event_rename::semantic_name(rename_kind, new_name) else {
        return Answer::Available(None);
    };
    let Some(bridge) = initialized_bridge(bridge) else {
        return Answer::Unavailable;
    };
    let Some(document) = corsa_support::open_canonical_virtual_project_document(ctx, bridge).await
    else {
        return Answer::Unavailable;
    };
    let Some((line, character)) = event_rename::semantic_position(ctx, &document)
        .or_else(|| corsa_support::canonical_source_offset_to_position(&document, ctx.offset))
    else {
        return Answer::Unavailable;
    };
    let response = match bridge
        .rename(&document.request_uri, line, character, &semantic_name)
        .await
    {
        Ok(response) => response,
        Err(_) => return Answer::Unavailable,
    };
    let Some(response) = response else {
        return Answer::Available(None);
    };
    let Ok(response) = serde_json::from_value::<WorkspaceEdit>(response) else {
        return Answer::Available(None);
    };
    let mut linked = linked_positions(&document, &response);
    if matches!(rename_kind, Some(event_rename::RenameKind::Model)) {
        linked.extend(event_rename::model_linked_positions(
            ctx, &document, &response,
        ));
    }
    let mut mapped = corsa_support::map_canonical_corsa_workspace_edit(ctx, &document, response)
        .into_iter()
        .collect::<Vec<_>>();

    for position in linked {
        let Ok(Some(extra)) = bridge
            .rename(
                &position.request_uri,
                position.line,
                position.character,
                &semantic_name,
            )
            .await
        else {
            return Answer::Available(None);
        };
        let Ok(extra) = serde_json::from_value(extra) else {
            return Answer::Available(None);
        };
        let Some(extra) = corsa_support::map_canonical_corsa_workspace_edit(ctx, &document, extra)
        else {
            return Answer::Available(None);
        };
        mapped.push(extra);
    }
    if let Some(styles) = style_workspace_edit(ctx, &mapped, new_name) {
        mapped.push(styles);
    }

    if let Some(kind) = rename_kind {
        for edit in &mut mapped {
            event_rename::rewrite_edits(ctx, edit, &semantic_name, kind);
        }
    }

    Answer::Available(corsa_support::merge_canonical_workspace_edits(mapped))
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
