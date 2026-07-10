#![allow(clippy::disallowed_macros, clippy::disallowed_methods)]

use tower_lsp::lsp_types::{
    DocumentChangeOperation, DocumentChanges, OneOf, OptionalVersionedTextDocumentIdentifier,
    RenameFile, ResourceOp, TextDocumentEdit, TextEdit, Url, WorkspaceEdit,
};

use super::merge_workspace_edits;

fn text_edit(new_text: &str) -> TextEdit {
    TextEdit {
        range: Default::default(),
        new_text: new_text.to_string(),
    }
}

#[test]
fn merges_changes_into_document_changes() {
    let base_uri = Url::parse("file:///base.vue").unwrap();
    let overlay_uri = Url::parse("file:///overlay.vue").unwrap();
    let merged = merge_workspace_edits(
        Some(WorkspaceEdit {
            changes: None,
            document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                text_document: OptionalVersionedTextDocumentIdentifier {
                    uri: base_uri.clone(),
                    version: None,
                },
                edits: vec![OneOf::Left(text_edit("from-corsa"))],
            }])),
            change_annotations: None,
        }),
        Some(WorkspaceEdit {
            changes: Some(std::collections::HashMap::from([(
                overlay_uri.clone(),
                vec![text_edit("from-manual")],
            )])),
            document_changes: None,
            change_annotations: None,
        }),
    )
    .unwrap();

    assert!(merged.changes.is_none());

    let DocumentChanges::Edits(edits) = merged.document_changes.unwrap() else {
        panic!("expected document edits");
    };

    assert_eq!(edits.len(), 2);
    assert!(edits.iter().any(|edit| edit.text_document.uri == base_uri));
    assert!(
        edits
            .iter()
            .any(|edit| edit.text_document.uri == overlay_uri)
    );
}

#[test]
fn prefers_overlay_document_changes_over_base_changes() {
    let base_uri = Url::parse("file:///base.vue").unwrap();
    let overlay_uri = Url::parse("file:///overlay.vue").unwrap();
    let merged = merge_workspace_edits(
        Some(WorkspaceEdit {
            changes: Some(std::collections::HashMap::from([(
                base_uri.clone(),
                vec![text_edit("from-base")],
            )])),
            document_changes: None,
            change_annotations: None,
        }),
        Some(WorkspaceEdit {
            changes: None,
            document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                text_document: OptionalVersionedTextDocumentIdentifier {
                    uri: overlay_uri.clone(),
                    version: None,
                },
                edits: vec![OneOf::Left(text_edit("from-overlay"))],
            }])),
            change_annotations: None,
        }),
    )
    .unwrap();

    assert!(merged.changes.is_none());

    let DocumentChanges::Edits(edits) = merged.document_changes.unwrap() else {
        panic!("expected document edits");
    };

    assert_eq!(edits.len(), 2);
    assert!(edits.iter().any(|edit| edit.text_document.uri == base_uri));
    assert!(
        edits
            .iter()
            .any(|edit| edit.text_document.uri == overlay_uri)
    );
}

#[test]
fn inserts_manual_edits_before_resource_operations() {
    let manual_uri = Url::parse("file:///manual.vue").unwrap();
    let merged = merge_workspace_edits(
        Some(WorkspaceEdit {
            changes: None,
            document_changes: Some(DocumentChanges::Operations(vec![
                DocumentChangeOperation::Op(ResourceOp::Rename(RenameFile {
                    old_uri: Url::parse("file:///old.vue").unwrap(),
                    new_uri: Url::parse("file:///new.vue").unwrap(),
                    options: None,
                    annotation_id: None,
                })),
            ])),
            change_annotations: None,
        }),
        Some(WorkspaceEdit {
            changes: Some(std::collections::HashMap::from([(
                manual_uri.clone(),
                vec![text_edit("from-manual")],
            )])),
            document_changes: None,
            change_annotations: None,
        }),
    )
    .unwrap();

    let DocumentChanges::Operations(operations) = merged.document_changes.unwrap() else {
        panic!("expected document operations");
    };

    assert!(
        matches!(operations.first(), Some(DocumentChangeOperation::Edit(edit)) if edit.text_document.uri == manual_uri)
    );
    assert!(matches!(
        operations.get(1),
        Some(DocumentChangeOperation::Op(ResourceOp::Rename(_)))
    ));
}
