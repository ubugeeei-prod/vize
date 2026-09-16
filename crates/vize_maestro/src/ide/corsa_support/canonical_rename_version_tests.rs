use tower_lsp::lsp_types::{
    CreateFile, DocumentChangeOperation, DocumentChanges, OneOf,
    OptionalVersionedTextDocumentIdentifier, Position, Range, ResourceOp, TextDocumentEdit,
    TextEdit, Url, WorkspaceEdit,
};

use super::merge_canonical_workspace_edits;

fn document_edit(uri: &Url, version: Option<i32>, line: u32) -> TextDocumentEdit {
    TextDocumentEdit {
        text_document: OptionalVersionedTextDocumentIdentifier {
            uri: uri.clone(),
            version,
        },
        edits: vec![OneOf::Left(TextEdit {
            range: Range::new(Position::new(line, 0), Position::new(line, 5)),
            new_text: "renamed".to_string(),
        })],
    }
}

fn workspace(edits: Vec<TextDocumentEdit>) -> WorkspaceEdit {
    WorkspaceEdit {
        document_changes: Some(DocumentChanges::Edits(edits)),
        ..WorkspaceEdit::default()
    }
}

fn operations(edits: Vec<TextDocumentEdit>) -> WorkspaceEdit {
    WorkspaceEdit {
        document_changes: Some(DocumentChanges::Operations(
            edits
                .into_iter()
                .map(DocumentChangeOperation::Edit)
                .collect(),
        )),
        ..WorkspaceEdit::default()
    }
}

#[test]
fn rejects_rename_edits_from_different_document_versions_in_either_query_order() {
    let uri = Url::parse("file:///workspace/Child.vue").unwrap();
    for versions in [[Some(1), Some(2)], [Some(2), Some(1)]] {
        let merged = merge_canonical_workspace_edits([
            workspace(vec![document_edit(&uri, versions[0], 1)]),
            workspace(vec![document_edit(&uri, versions[1], 3)]),
        ]);
        assert_eq!(
            merged, None,
            "must not apply edits from two revisions as one"
        );
    }
}

#[test]
fn conflicting_versions_are_rejected_in_every_container_and_query_order() {
    let uri = Url::parse("file:///workspace/dependency.ts").unwrap();
    for first in [workspace, operations] {
        for second in [workspace, operations] {
            for versions in [[Some(1), Some(2)], [Some(2), Some(1)]] {
                assert_eq!(
                    merge_canonical_workspace_edits([
                        first(vec![document_edit(&uri, versions[0], 1)]),
                        second(vec![document_edit(&uri, versions[1], 3)]),
                    ]),
                    None
                );
            }
        }
    }
}

#[test]
fn conflicting_versions_within_one_response_are_also_rejected() {
    let uri = Url::parse("file:///workspace/dependency.ts").unwrap();
    for container in [workspace, operations] {
        assert_eq!(
            merge_canonical_workspace_edits([container(vec![
                document_edit(&uri, Some(1), 1),
                document_edit(&uri, Some(2), 3),
            ])]),
            None
        );
    }
}

#[test]
fn known_versions_survive_unknown_results_in_every_container_and_query_order() {
    let uri = Url::parse("file:///workspace/dependency.ts").unwrap();
    for first in [workspace, operations] {
        for second in [workspace, operations] {
            for versions in [[None, Some(0)], [Some(0), None], [None, None]] {
                let merged = merge_canonical_workspace_edits([
                    first(vec![document_edit(&uri, versions[0], 1)]),
                    second(vec![document_edit(&uri, versions[1], 3)]),
                ])
                .unwrap();
                let edits = match merged.document_changes.unwrap() {
                    DocumentChanges::Edits(edits) => edits,
                    DocumentChanges::Operations(operations) => operations
                        .into_iter()
                        .map(|op| {
                            let DocumentChangeOperation::Edit(edit) = op else {
                                panic!("unexpected resource operation");
                            };
                            edit
                        })
                        .collect(),
                };
                let expected_version = versions[0].or(versions[1]);
                for edit in &edits {
                    assert_eq!(edit.text_document.version, expected_version);
                    assert_eq!(edit.text_document.uri, uri);
                }
                assert_eq!(
                    edits
                        .into_iter()
                        .flat_map(|edit| edit.edits)
                        .collect::<Vec<_>>(),
                    [
                        document_edit(&uri, expected_version, 1).edits[0].clone(),
                        document_edit(&uri, expected_version, 3).edits[0].clone()
                    ]
                );
            }
        }
    }
}

#[test]
fn promoted_plain_edits_keep_the_known_version_and_resource_operation_order() {
    let uri = Url::parse("file:///workspace/dependency.ts").unwrap();
    let create = DocumentChangeOperation::Op(ResourceOp::Create(CreateFile {
        uri: Url::parse("file:///workspace/new.ts").unwrap(),
        options: None,
        annotation_id: None,
    }));
    let first = WorkspaceEdit {
        document_changes: Some(DocumentChanges::Operations(vec![
            create.clone(),
            DocumentChangeOperation::Edit(document_edit(&uri, Some(9), 1)),
        ])),
        ..WorkspaceEdit::default()
    };
    let plain = TextEdit {
        range: Range::new(Position::new(3, 0), Position::new(3, 5)),
        new_text: "renamed".to_string(),
    };
    let second = WorkspaceEdit {
        changes: Some([(uri.clone(), vec![plain])].into()),
        ..WorkspaceEdit::default()
    };
    let merged = merge_canonical_workspace_edits([first, second]).unwrap();
    assert_eq!(merged.changes, None);
    assert_eq!(
        merged.document_changes,
        Some(DocumentChanges::Operations(vec![
            create,
            DocumentChangeOperation::Edit(document_edit(&uri, Some(9), 1)),
            DocumentChangeOperation::Edit(document_edit(&uri, Some(9), 3)),
        ]))
    );
}

#[test]
fn equal_versions_keep_all_edits_and_the_version_guard() {
    let uri = Url::parse("file:///workspace/Child.vue").unwrap();
    let merged = merge_canonical_workspace_edits([
        workspace(vec![document_edit(&uri, Some(4), 3)]),
        workspace(vec![document_edit(&uri, Some(4), 1)]),
    ])
    .unwrap();
    let Some(DocumentChanges::Edits(edits)) = merged.document_changes else {
        panic!("expected document edits");
    };
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].text_document.version, Some(4));
    assert_eq!(
        edits[0].edits,
        [
            document_edit(&uri, Some(4), 1).edits[0].clone(),
            document_edit(&uri, Some(4), 3).edits[0].clone()
        ]
    );
}

#[test]
fn versions_are_checked_per_file_not_across_the_project() {
    let a = Url::parse("file:///workspace/A.vue").unwrap();
    let b = Url::parse("file:///workspace/B.vue").unwrap();
    let merged = merge_canonical_workspace_edits([
        workspace(vec![document_edit(&a, Some(1), 1)]),
        workspace(vec![document_edit(&b, Some(2), 1)]),
    ])
    .unwrap();
    let Some(DocumentChanges::Edits(edits)) = merged.document_changes else {
        panic!("expected document edits");
    };
    assert_eq!(edits.len(), 2);
    assert_eq!(edits[0].text_document.version, Some(1));
    assert_eq!(edits[1].text_document.version, Some(2));
}
