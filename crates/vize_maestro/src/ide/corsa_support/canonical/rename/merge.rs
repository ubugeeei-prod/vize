use std::collections::HashMap;

use tower_lsp::lsp_types::{
    AnnotatedTextEdit, DocumentChangeOperation, DocumentChanges, OneOf,
    OptionalVersionedTextDocumentIdentifier, TextDocumentEdit, TextEdit, Url, WorkspaceEdit,
};

use super::super::super::rename_merge::order_edits_by_position;
use super::{annotatable_identity, push_text_edit, workspace_edit_is_empty};

pub(crate) fn merge_canonical_workspace_edits(
    edits: impl IntoIterator<Item = WorkspaceEdit>,
) -> Option<WorkspaceEdit> {
    let mut changes = HashMap::new();
    let mut document_changes = None;
    let mut change_annotations = HashMap::new();
    let mut versions = HashMap::new();

    for mut edit in edits {
        for (uri, edits) in edit.changes.take().unwrap_or_default() {
            for edit in edits {
                push_text_edit(&mut changes, uri.clone(), edit);
            }
        }
        if let Some(mut incoming) = edit.document_changes.take() {
            // Linked rename queries can observe different dependency revisions.
            // Never label edits from conflicting snapshots with one version.
            visit_documents(&mut incoming, |document| {
                if let Some(version) = document.text_document.version {
                    let known = versions
                        .entry(document.text_document.uri.clone())
                        .or_insert(version);
                    if *known != version {
                        return None;
                    }
                }
                Some(())
            })?;
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
        visit_documents(document_changes, |document| {
            document.text_document.version = versions.get(&document.text_document.uri).copied();
            Some(())
        })?;
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

fn visit_documents(
    changes: &mut DocumentChanges,
    mut visit: impl FnMut(&mut TextDocumentEdit) -> Option<()>,
) -> Option<()> {
    match changes {
        DocumentChanges::Edits(edits) => {
            for edit in edits {
                visit(edit)?;
            }
        }
        DocumentChanges::Operations(operations) => {
            for operation in operations {
                if let DocumentChangeOperation::Edit(edit) = operation {
                    visit(edit)?;
                }
            }
        }
    }
    Some(())
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
