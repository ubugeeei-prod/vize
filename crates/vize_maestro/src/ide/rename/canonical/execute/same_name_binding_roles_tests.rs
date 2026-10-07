use std::collections::HashMap;

use tower_lsp::lsp_types::{
    AnnotatedTextEdit, ChangeAnnotation, DocumentChanges, Location, OneOf,
    OptionalVersionedTextDocumentIdentifier, TextDocumentEdit, TextEdit, Url, WorkspaceEdit,
};

use super::{coherent, rewrite_roles};
use crate::ide::{DiagnosticService, IdeContext, corsa_support::CanonicalVirtualDocument};
use crate::server::ServerState;

#[test]
fn selected_key_and_value_roles_expand_once_without_losing_versions_or_annotations() {
    let source = "<script setup lang=\"ts\">defineProps<{ id: string }>();</script>\r\n<template>😀<Recursive :id.camel /><input :id /></template>";
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/Recursive.vue").expect("URI");
    let ctx = IdeContext::testing(&state, &uri, 0, source.into());
    let document = CanonicalVirtualDocument {
        source_uri: uri.clone(),
        request_uri: "file:///workspace/Recursive.vue.ts".into(),
        virtual_result: DiagnosticService::generate_virtual_ts(&uri, source, false, false)
            .expect("generated document"),
        dependencies: Vec::new(),
        materialized_sources: Vec::new(),
        session_project_roots: Vec::new(),
        source_catalogs: Vec::new(),
    };
    let start = source.find(":id.camel").expect("shorthand");
    let range = |start, end| super::super::super::event_rename::offset_range(source, start..end);
    let argument = Location::new(uri.clone(), range(start + 1, start + 3));
    let annotations = HashMap::from([(
        "selected-native-role".into(),
        ChangeAnnotation {
            label: "Recursive prop".into(),
            needs_confirmation: Some(false),
            description: Some("One native symbol transaction".into()),
        },
    )]);
    for (values, replacement) in [
        (Vec::new(), ":field.camel=\"id\""),
        (vec![argument.clone()], ":field.camel=\"field\""),
    ] {
        let mut edit = WorkspaceEdit {
            document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                text_document: OptionalVersionedTextDocumentIdentifier {
                    uri: uri.clone(),
                    version: Some(7),
                },
                edits: vec![OneOf::Right(AnnotatedTextEdit {
                    text_edit: TextEdit {
                        range: argument.range,
                        new_text: "field".into(),
                    },
                    annotation_id: "selected-native-role".into(),
                })],
            }])),
            change_annotations: Some(annotations.clone()),
            ..Default::default()
        };
        assert!(rewrite_roles(
            &ctx,
            &document,
            &mut edit,
            "field",
            std::slice::from_ref(&argument),
            &values,
        ));
        assert!(coherent(&edit));
        assert_eq!(edit.change_annotations, Some(annotations.clone()));
        assert_eq!(
            edit.document_changes,
            Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                text_document: OptionalVersionedTextDocumentIdentifier {
                    uri: uri.clone(),
                    version: Some(7)
                },
                edits: vec![OneOf::Right(AnnotatedTextEdit {
                    text_edit: TextEdit {
                        range: range(start, start + ":id.camel".len()),
                        new_text: replacement.into()
                    },
                    annotation_id: "selected-native-role".into(),
                })],
            }]))
        );
    }
}
