use std::collections::HashMap;

use tower_lsp::lsp_types::{
    AnnotatedTextEdit, DocumentChangeOperation, DocumentChanges, OneOf, TextEdit, WorkspaceEdit,
};

use crate::ide::IdeContext;
use crate::virtual_code::VirtualDocument;

pub(crate) fn map_corsa_workspace_edit(
    ctx: &IdeContext<'_>,
    mut edit: WorkspaceEdit,
) -> Option<WorkspaceEdit> {
    if let Some(changes) = edit.changes.take() {
        let mut mapped_changes = HashMap::with_capacity(changes.len());

        for (uri, edits) in changes {
            if let Some(current_doc) = super::match_current_virtual_document(ctx, uri.as_str()) {
                let entry = mapped_changes
                    .entry(ctx.uri.clone())
                    .or_insert_with(Vec::new);
                entry.extend(
                    edits
                        .into_iter()
                        .filter_map(|edit| map_text_edit(ctx, current_doc.document(), edit)),
                );
            } else {
                mapped_changes.insert(uri, edits);
            }
        }

        if !mapped_changes.is_empty() {
            edit.changes = Some(mapped_changes);
        }
    }

    if let Some(document_changes) = edit.document_changes.take() {
        let mapped_document_changes = match document_changes {
            DocumentChanges::Edits(edits) => {
                let edits = edits
                    .into_iter()
                    .filter_map(|edit| map_document_edit(ctx, edit))
                    .collect::<Vec<_>>();

                if edits.is_empty() {
                    None
                } else {
                    Some(DocumentChanges::Edits(edits))
                }
            }
            DocumentChanges::Operations(operations) => {
                let operations = operations
                    .into_iter()
                    .filter_map(|operation| map_document_change_operation(ctx, operation))
                    .collect::<Vec<_>>();

                if operations.is_empty() {
                    None
                } else {
                    Some(DocumentChanges::Operations(operations))
                }
            }
        };

        if let Some(document_changes) = mapped_document_changes {
            edit.document_changes = Some(document_changes);
        }
    }

    if workspace_edit_is_empty(&edit) {
        None
    } else {
        Some(edit)
    }
}

fn workspace_edit_is_empty(edit: &WorkspaceEdit) -> bool {
    let changes_empty = edit
        .changes
        .as_ref()
        .is_none_or(|changes| changes.values().all(Vec::is_empty));
    let document_changes_empty =
        edit.document_changes
            .as_ref()
            .is_none_or(|changes| match changes {
                DocumentChanges::Edits(edits) => edits.is_empty(),
                DocumentChanges::Operations(operations) => operations.is_empty(),
            });

    changes_empty && document_changes_empty
}

fn map_document_change_operation(
    ctx: &IdeContext<'_>,
    operation: DocumentChangeOperation,
) -> Option<DocumentChangeOperation> {
    match operation {
        DocumentChangeOperation::Edit(edit) => {
            map_document_edit(ctx, edit).map(DocumentChangeOperation::Edit)
        }
        DocumentChangeOperation::Op(op) => Some(DocumentChangeOperation::Op(op)),
    }
}

fn map_document_edit(
    ctx: &IdeContext<'_>,
    mut edit: tower_lsp::lsp_types::TextDocumentEdit,
) -> Option<tower_lsp::lsp_types::TextDocumentEdit> {
    let current_doc = super::match_current_virtual_document(ctx, edit.text_document.uri.as_str());

    if let Some(current_doc) = current_doc {
        edit.text_document.uri = ctx.uri.clone();
        edit.edits = edit
            .edits
            .into_iter()
            .filter_map(|entry| match entry {
                OneOf::Left(text_edit) => {
                    map_text_edit(ctx, current_doc.document(), text_edit).map(OneOf::Left)
                }
                OneOf::Right(annotated) => {
                    map_annotated_text_edit(ctx, current_doc.document(), annotated)
                        .map(OneOf::Right)
                }
            })
            .collect();
    }

    if edit.edits.is_empty() {
        None
    } else {
        Some(edit)
    }
}

fn map_annotated_text_edit(
    ctx: &IdeContext<'_>,
    document: &VirtualDocument,
    mut edit: AnnotatedTextEdit,
) -> Option<AnnotatedTextEdit> {
    edit.text_edit = map_text_edit(ctx, document, edit.text_edit)?;
    Some(edit)
}

fn map_text_edit(
    ctx: &IdeContext<'_>,
    document: &VirtualDocument,
    mut edit: TextEdit,
) -> Option<TextEdit> {
    edit.range = super::map_virtual_range(ctx, document, &edit.range)?;
    Some(edit)
}
