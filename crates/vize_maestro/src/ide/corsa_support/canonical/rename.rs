use std::collections::HashMap;

use tower_lsp::lsp_types::{
    AnnotatedTextEdit, DocumentChangeOperation, DocumentChanges, OneOf,
    OptionalVersionedTextDocumentIdentifier, PrepareRenameResponse, ResourceOp, TextDocumentEdit,
    TextEdit, Url, WorkspaceEdit,
};
use vize_canon::{LspLocation, LspPosition, LspRange};

use super::super::rename_merge::order_edits_by_position;
use super::{CanonicalVirtualDocument, is_canonical_vue_virtual_uri};
use crate::ide::IdeContext;

pub(crate) fn map_canonical_prepare_rename(
    ctx: &IdeContext<'_>,
    document: &CanonicalVirtualDocument,
    response: PrepareRenameResponse,
) -> Option<PrepareRenameResponse> {
    match response {
        PrepareRenameResponse::Range(range)
        | PrepareRenameResponse::RangeWithPlaceholder { range, .. } => {
            super::map_canonical_lsp_range(ctx, document, &to_canonical_range(range))
                .map(PrepareRenameResponse::Range)
        }
        PrepareRenameResponse::DefaultBehavior { default_behavior } => {
            Some(PrepareRenameResponse::DefaultBehavior { default_behavior })
        }
    }
}

pub(crate) fn map_canonical_corsa_workspace_edit(
    ctx: &IdeContext<'_>,
    document: &CanonicalVirtualDocument,
    mut edit: WorkspaceEdit,
) -> Option<WorkspaceEdit> {
    if let Some(changes) = edit.changes.take() {
        let mut mapped = HashMap::new();
        for (uri, edits) in changes {
            for edit in edits {
                if let Some((uri, edit)) = map_text_edit(ctx, document, &uri, edit) {
                    push_text_edit(&mut mapped, uri, edit);
                }
            }
        }
        edit.changes = (!mapped.is_empty()).then_some(mapped);
    }

    if let Some(changes) = edit.document_changes.take() {
        edit.document_changes = map_document_changes(ctx, document, changes);
    }

    (!workspace_edit_is_empty(&edit)).then_some(edit)
}

pub(crate) fn merge_canonical_workspace_edits(
    edits: impl IntoIterator<Item = WorkspaceEdit>,
) -> Option<WorkspaceEdit> {
    let mut changes = HashMap::new();
    let mut document_changes = None;
    let mut change_annotations = HashMap::new();

    for mut edit in edits {
        for (uri, edits) in edit.changes.take().unwrap_or_default() {
            for edit in edits {
                push_text_edit(&mut changes, uri.clone(), edit);
            }
        }
        if let Some(incoming) = edit.document_changes.take() {
            merge_document_change_sets(&mut document_changes, incoming);
        }
        for (id, annotation) in edit.change_annotations.take().unwrap_or_default() {
            if change_annotations
                .get(&id)
                .is_some_and(|existing| existing != &annotation)
            {
                return None;
            }
            change_annotations.insert(id, annotation);
        }
    }

    if let Some(document_changes) = document_changes.as_mut() {
        promote_plain_changes(document_changes, std::mem::take(&mut changes));
    }
    let edit = WorkspaceEdit {
        changes: (!changes.is_empty()).then_some(changes),
        document_changes,
        change_annotations: (!change_annotations.is_empty()).then_some(change_annotations),
    };
    // Each incoming edit answers one canonical query - the symbol under the
    // cursor, then every position linked to it, then the style sweep - so
    // concatenating them leaves a template-side rename reporting its own
    // occurrence before the declaration that sits above it.
    (!workspace_edit_is_empty(&edit)).then(|| order_edits_by_position(edit))
}

fn merge_document_change_sets(current: &mut Option<DocumentChanges>, incoming: DocumentChanges) {
    let Some(current) = current else {
        *current = Some(incoming);
        return;
    };
    match (current, incoming) {
        (DocumentChanges::Edits(current), DocumentChanges::Edits(incoming)) => {
            for edit in incoming {
                merge_document_edit(current, edit);
            }
        }
        (DocumentChanges::Operations(current), DocumentChanges::Operations(mut incoming)) => {
            current.append(&mut incoming);
        }
        (current @ DocumentChanges::Edits(_), DocumentChanges::Operations(mut incoming)) => {
            let DocumentChanges::Edits(edits) =
                std::mem::replace(current, DocumentChanges::Operations(Vec::new()))
            else {
                unreachable!();
            };
            let DocumentChanges::Operations(current) = current else {
                unreachable!();
            };
            current.extend(edits.into_iter().map(DocumentChangeOperation::Edit));
            current.append(&mut incoming);
        }
        (DocumentChanges::Operations(current), DocumentChanges::Edits(incoming)) => {
            current.extend(incoming.into_iter().map(DocumentChangeOperation::Edit));
        }
    }
}

fn promote_plain_changes(changes: &mut DocumentChanges, plain: HashMap<Url, Vec<TextEdit>>) {
    for (uri, edits) in plain {
        let edit = TextDocumentEdit {
            text_document: OptionalVersionedTextDocumentIdentifier { uri, version: None },
            edits: edits.into_iter().map(OneOf::Left).collect(),
        };
        match changes {
            DocumentChanges::Edits(changes) => merge_document_edit(changes, edit),
            DocumentChanges::Operations(changes) => {
                changes.push(DocumentChangeOperation::Edit(edit));
            }
        }
    }
}

fn merge_document_edit(edits: &mut Vec<TextDocumentEdit>, incoming: TextDocumentEdit) {
    let Some(existing) = edits
        .iter_mut()
        .find(|edit| edit.text_document.uri == incoming.text_document.uri)
    else {
        edits.push(incoming);
        return;
    };
    for edit in incoming.edits {
        push_annotatable_edit_to(existing, edit);
    }
}

fn push_annotatable_edit_to(
    document: &mut TextDocumentEdit,
    edit: OneOf<TextEdit, AnnotatedTextEdit>,
) {
    let (range, new_text) = annotatable_identity(&edit);
    if !document.edits.iter().any(|existing| {
        let (existing_range, existing_text) = annotatable_identity(existing);
        existing_range == range && existing_text == new_text
    }) {
        document.edits.push(edit);
    }
}

fn map_document_changes(
    ctx: &IdeContext<'_>,
    document: &CanonicalVirtualDocument,
    changes: DocumentChanges,
) -> Option<DocumentChanges> {
    match changes {
        DocumentChanges::Edits(edits) => {
            let mapped = edits
                .into_iter()
                .flat_map(|edit| map_document_edit(ctx, document, edit))
                .collect::<Vec<_>>();
            (!mapped.is_empty()).then_some(DocumentChanges::Edits(mapped))
        }
        DocumentChanges::Operations(operations) => {
            let mut mapped = Vec::new();
            for operation in operations {
                match operation {
                    DocumentChangeOperation::Edit(edit) => mapped.extend(
                        map_document_edit(ctx, document, edit)
                            .into_iter()
                            .map(DocumentChangeOperation::Edit),
                    ),
                    DocumentChangeOperation::Op(operation)
                        if resource_operation_is_authored(document, &operation) =>
                    {
                        mapped.push(DocumentChangeOperation::Op(operation));
                    }
                    DocumentChangeOperation::Op(_) => {}
                }
            }
            (!mapped.is_empty()).then_some(DocumentChanges::Operations(mapped))
        }
    }
}

fn map_document_edit(
    ctx: &IdeContext<'_>,
    document: &CanonicalVirtualDocument,
    edit: TextDocumentEdit,
) -> Vec<TextDocumentEdit> {
    let original = edit.text_document;
    let mut groups: Vec<(Url, Vec<OneOf<TextEdit, AnnotatedTextEdit>>)> = Vec::new();

    for entry in edit.edits {
        let mapped = match entry {
            OneOf::Left(edit) => map_text_edit(ctx, document, &original.uri, edit)
                .map(|(uri, edit)| (uri, OneOf::Left(edit))),
            OneOf::Right(AnnotatedTextEdit {
                text_edit,
                annotation_id,
            }) => map_text_edit(ctx, document, &original.uri, text_edit).map(|(uri, text_edit)| {
                (
                    uri,
                    OneOf::Right(AnnotatedTextEdit {
                        text_edit,
                        annotation_id,
                    }),
                )
            }),
        };
        if let Some((uri, entry)) = mapped {
            push_annotatable_edit(&mut groups, uri, entry);
        }
    }

    groups
        .into_iter()
        .map(|(uri, edits)| TextDocumentEdit {
            text_document: OptionalVersionedTextDocumentIdentifier {
                version: if uri == original.uri {
                    original.version
                } else {
                    None
                },
                uri,
            },
            edits,
        })
        .collect()
}

fn map_text_edit(
    ctx: &IdeContext<'_>,
    document: &CanonicalVirtualDocument,
    uri: &Url,
    mut edit: TextEdit,
) -> Option<(Url, TextEdit)> {
    let location = super::map_canonical_corsa_location(
        ctx,
        document,
        &LspLocation {
            uri: uri.to_string(),
            range: to_canonical_range(edit.range),
        },
    )?;
    edit.range = location.range;
    Some((location.uri, edit))
}

fn push_text_edit(changes: &mut HashMap<Url, Vec<TextEdit>>, uri: Url, edit: TextEdit) {
    let edits = changes.entry(uri).or_default();
    if !edits
        .iter()
        .any(|existing| existing.range == edit.range && existing.new_text == edit.new_text)
    {
        edits.push(edit);
    }
}

fn push_annotatable_edit(
    groups: &mut Vec<(Url, Vec<OneOf<TextEdit, AnnotatedTextEdit>>)>,
    uri: Url,
    edit: OneOf<TextEdit, AnnotatedTextEdit>,
) {
    let edits = if let Some((_, edits)) = groups.iter_mut().find(|(existing, _)| *existing == uri) {
        edits
    } else {
        groups.push((uri, Vec::new()));
        &mut groups.last_mut().expect("inserted group").1
    };
    let (range, new_text) = annotatable_identity(&edit);
    if !edits.iter().any(|existing| {
        let (existing_range, existing_text) = annotatable_identity(existing);
        existing_range == range && existing_text == new_text
    }) {
        edits.push(edit);
    }
}

fn annotatable_identity(
    edit: &OneOf<TextEdit, AnnotatedTextEdit>,
) -> (tower_lsp::lsp_types::Range, &str) {
    match edit {
        OneOf::Left(edit) => (edit.range, edit.new_text.as_str()),
        OneOf::Right(edit) => (edit.text_edit.range, edit.text_edit.new_text.as_str()),
    }
}

fn resource_operation_is_authored(
    document: &CanonicalVirtualDocument,
    operation: &ResourceOp,
) -> bool {
    let safe = |uri: &Url| {
        !super::is_private_materialized_uri(document, uri.as_str())
            && (!is_canonical_vue_virtual_uri(uri)
                || uri.to_file_path().is_ok_and(|path| path.is_file()))
    };
    match operation {
        ResourceOp::Create(operation) => safe(&operation.uri),
        ResourceOp::Rename(operation) => safe(&operation.old_uri) && safe(&operation.new_uri),
        ResourceOp::Delete(operation) => safe(&operation.uri),
    }
}

fn to_canonical_range(range: tower_lsp::lsp_types::Range) -> LspRange {
    LspRange {
        start: LspPosition {
            line: range.start.line,
            character: range.start.character,
        },
        end: LspPosition {
            line: range.end.line,
            character: range.end.character,
        },
    }
}

fn workspace_edit_is_empty(edit: &WorkspaceEdit) -> bool {
    edit.changes
        .as_ref()
        .is_none_or(|changes| changes.values().all(Vec::is_empty))
        && edit
            .document_changes
            .as_ref()
            .is_none_or(|changes| match changes {
                DocumentChanges::Edits(edits) => edits.is_empty(),
                DocumentChanges::Operations(operations) => operations.is_empty(),
            })
}
