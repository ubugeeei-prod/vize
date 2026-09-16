use std::collections::HashMap;

use tower_lsp::lsp_types::{
    AnnotatedTextEdit, DocumentChangeOperation, DocumentChanges, OneOf,
    OptionalVersionedTextDocumentIdentifier, PrepareRenameResponse, ResourceOp, TextDocumentEdit,
    TextEdit, Url, WorkspaceEdit,
};
use vize_canon::{LspLocation, LspPosition, LspRange};

use super::{CanonicalVirtualDocument, is_canonical_vue_virtual_uri};
use crate::ide::IdeContext;

mod merge;
pub(crate) use merge::merge_canonical_workspace_edits;

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
