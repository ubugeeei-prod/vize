use tower_lsp::lsp_types::{
    AnnotatedTextEdit, CreateFile, DocumentChangeOperation, DocumentChanges, OneOf,
    OptionalVersionedTextDocumentIdentifier, Position, Range, ResourceOp, TextDocumentEdit,
    TextEdit, Url, WorkspaceEdit,
};

use super::canonical::rename::{
    map_canonical_corsa_workspace_edit, merge_canonical_workspace_edits,
};
use super::canonical_dependency_tests::{
    generated_offset_for_source, host_document, mapped_document,
};
use crate::ide::{IdeContext, offset_to_position, position_to_offset};
use crate::server::ServerState;

#[test]
fn maps_annotated_document_edits_and_drops_synthetic_resource_operations() {
    let project = tempfile::TempDir::new().expect("temp project");
    let host_path = project.path().join("Host.vue");
    let child_path = project.path().join("Child View.vue");
    let host_uri = Url::from_file_path(&host_path).expect("host uri");
    let child_uri = Url::from_file_path(&child_path).expect("child uri");
    let host_source = "<template><main /></template>";
    let child_source = "<script setup>const emoji = '💥'; const shared = 1</script>";
    std::fs::write(&host_path, host_source).expect("host");
    std::fs::write(&child_path, child_source).expect("child");

    let state = ServerState::new();
    state.documents.open(
        host_uri.clone(),
        host_source.to_string(),
        1,
        "vue".to_string(),
    );
    let ctx = IdeContext::new(&state, &host_uri, 1).expect("host context");
    let dependency = mapped_document(&child_uri, child_source);
    let source_start = child_source.find("shared").expect("source token");
    let generated_start = generated_offset_for_source(&dependency, source_start);
    let generated_end = generated_offset_for_source(&dependency, source_start + "shared".len());
    let (start_line, start_character) =
        offset_to_position(&dependency.virtual_result.code, generated_start);
    let (end_line, end_character) =
        offset_to_position(&dependency.virtual_result.code, generated_end);
    let request_uri = Url::parse(&dependency.request_uri).expect("request uri");
    let annotated = AnnotatedTextEdit {
        text_edit: TextEdit {
            range: Range::new(
                Position::new(start_line, start_character),
                Position::new(end_line, end_character),
            ),
            new_text: "renamed".to_string(),
        },
        annotation_id: "rename-symbol".to_string(),
    };
    let mut host = host_document(&host_uri, host_source);
    host.dependencies.push(dependency);

    let mapped = map_canonical_corsa_workspace_edit(
        &ctx,
        &host,
        WorkspaceEdit {
            changes: None,
            document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                text_document: OptionalVersionedTextDocumentIdentifier {
                    uri: request_uri,
                    version: Some(9),
                },
                edits: vec![OneOf::Right(annotated)],
            }])),
            change_annotations: None,
        },
    )
    .expect("mapped workspace edit");
    let Some(DocumentChanges::Edits(edits)) = mapped.document_changes else {
        panic!("expected document edits");
    };
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].text_document.uri, child_uri);
    assert_eq!(edits[0].text_document.version, None);
    let OneOf::Right(edit) = &edits[0].edits[0] else {
        panic!("expected annotated edit");
    };
    assert_eq!(edit.annotation_id, "rename-symbol");
    let mapped_start = position_to_offset(
        child_source,
        edit.text_edit.range.start.line,
        edit.text_edit.range.start.character,
    )
    .unwrap();
    let mapped_end = position_to_offset(
        child_source,
        edit.text_edit.range.end.line,
        edit.text_edit.range.end.character,
    )
    .unwrap();
    assert_eq!(&child_source[mapped_start..mapped_end], "shared");

    let synthetic_uri = Url::from_file_path(child_path.with_extension("vue.ts")).unwrap();
    assert!(
        map_canonical_corsa_workspace_edit(
            &ctx,
            &host,
            WorkspaceEdit {
                changes: None,
                document_changes: Some(DocumentChanges::Operations(vec![
                    DocumentChangeOperation::Op(ResourceOp::Create(CreateFile {
                        uri: synthetic_uri,
                        options: None,
                        annotation_id: None,
                    })),
                ])),
                change_annotations: None,
            },
        )
        .is_none(),
        "synthetic resource operations must fail closed",
    );
}

#[test]
fn merges_plain_and_annotated_canonical_rename_edits_without_forking_containers() {
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    let plain = TextEdit {
        range: Range::new(Position::new(1, 2), Position::new(1, 8)),
        new_text: "renamed".to_string(),
    };
    let annotated = AnnotatedTextEdit {
        text_edit: TextEdit {
            range: Range::new(Position::new(3, 4), Position::new(3, 10)),
            new_text: "renamed".to_string(),
        },
        annotation_id: "template-use".to_string(),
    };

    let merged = merge_canonical_workspace_edits([
        WorkspaceEdit {
            changes: Some(std::collections::HashMap::from([(
                uri.clone(),
                vec![plain.clone()],
            )])),
            document_changes: None,
            change_annotations: None,
        },
        WorkspaceEdit {
            changes: None,
            document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                text_document: OptionalVersionedTextDocumentIdentifier {
                    uri: uri.clone(),
                    version: None,
                },
                edits: vec![OneOf::Right(annotated.clone())],
            }])),
            change_annotations: None,
        },
    ])
    .expect("merged edit");

    assert!(
        merged.changes.is_none(),
        "clients must see one edit container"
    );
    let Some(DocumentChanges::Edits(edits)) = merged.document_changes else {
        panic!("expected document edits");
    };
    assert_eq!(edits.len(), 1);
    assert_eq!(
        edits[0].edits,
        [OneOf::Left(plain), OneOf::Right(annotated)],
        "the merged container must read in document order",
    );
}

/// Regression for the create-vue editor range oracle: a template-side rename
/// answers the occurrence under the cursor first and picks up the declaration
/// above it from a linked position, so the concatenated edits used to reach the
/// client as `[8:9-16, 4:6-13]`.
#[test]
fn orders_canonical_rename_edits_by_position() {
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    let template_use = TextEdit {
        range: Range::new(Position::new(8, 9), Position::new(8, 16)),
        new_text: "twice".to_string(),
    };
    let declaration = TextEdit {
        range: Range::new(Position::new(4, 6), Position::new(4, 13)),
        new_text: "twice".to_string(),
    };

    let merged = merge_canonical_workspace_edits([
        WorkspaceEdit {
            changes: Some(std::collections::HashMap::from([(
                uri.clone(),
                vec![template_use.clone()],
            )])),
            document_changes: None,
            change_annotations: None,
        },
        WorkspaceEdit {
            changes: Some(std::collections::HashMap::from([(
                uri.clone(),
                vec![declaration.clone()],
            )])),
            document_changes: None,
            change_annotations: None,
        },
    ])
    .expect("merged edit");

    assert_eq!(
        merged.changes.expect("changes")[&uri],
        [declaration, template_use],
    );
}
