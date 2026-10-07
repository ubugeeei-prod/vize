use std::collections::HashMap;

use tower_lsp::lsp_types::{
    AnnotatedTextEdit, DocumentChanges, OneOf, OptionalVersionedTextDocumentIdentifier,
    TextDocumentEdit, TextEdit, Url, WorkspaceEdit,
};

use super::{coherent, is_binding_declaration, rewrite};
use crate::ide::{DiagnosticService, IdeContext, corsa_support::CanonicalVirtualDocument};
use crate::server::ServerState;

fn document(uri: &Url, source: &str) -> CanonicalVirtualDocument {
    CanonicalVirtualDocument {
        source_uri: uri.clone(),
        request_uri: "file:///workspace/App.vue.ts".into(),
        virtual_result: DiagnosticService::generate_virtual_ts(uri, source, false, false)
            .expect("generated document"),
        dependencies: Vec::new(),
        materialized_sources: Vec::new(),
        session_project_roots: Vec::new(),
        source_catalogs: Vec::new(),
    }
}

fn edit(source: &str, needle: &str, new_name: &str) -> TextEdit {
    let start = source.find(needle).unwrap();
    TextEdit {
        range: super::super::super::event_rename::offset_range(source, start..start + needle.len()),
        new_text: new_name.into(),
    }
}

#[test]
fn local_rename_expands_component_and_native_shorthands_with_modifiers() {
    let source = "<script setup>const label = 'Name'; const id = 'field';</script>\n<template><Child :label.camel /><input v-bind:id.prop /></template>";
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    let ctx = IdeContext::testing(&state, &uri, 0, source.into());
    for (needle, replacement) in [
        (":label.camel", ":label.camel=\"title\""),
        ("v-bind:id.prop", "v-bind:id.prop=\"title\""),
    ] {
        let argument = if needle.starts_with(':') {
            "label.camel"
        } else {
            "id.prop"
        };
        let start = source.find(needle).unwrap() + needle.find(argument).unwrap();
        let end = start + argument.split('.').next().unwrap().len();
        let mut response = WorkspaceEdit {
            changes: Some(HashMap::from([(
                uri.clone(),
                vec![TextEdit {
                    range: super::super::super::event_rename::offset_range(source, start..end),
                    new_text: "title".into(),
                }],
            )])),
            ..Default::default()
        };
        assert!(rewrite(
            &ctx,
            &document(&uri, source),
            &mut response,
            "title",
            false
        ));
        assert!(coherent(&response));
        assert_eq!(
            response.changes.unwrap()[&uri],
            [edit(source, needle, replacement)]
        );
    }
}

#[test]
fn prop_rename_keeps_the_camelized_implicit_value_and_complete_argument() {
    let source = "<script setup>import Child from './Child.vue'; const isOpened = true;</script>\n<template><Child :is-opened.camel /></template>";
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    let ctx = IdeContext::testing(&state, &uri, 0, source.into());
    let mut response = WorkspaceEdit {
        changes: Some(HashMap::from([(
            uri.clone(),
            vec![edit(source, "is-opened.camel", "visible")],
        )])),
        ..Default::default()
    };
    response.changes.as_mut().unwrap().get_mut(&uri).unwrap()[0]
        .range
        .end
        .character -= ".camel".len() as u32;
    assert!(rewrite(
        &ctx,
        &document(&uri, source),
        &mut response,
        "visible",
        true
    ));
    assert_eq!(
        response.changes.unwrap()[&uri],
        [edit(
            source,
            ":is-opened.camel",
            ":visible.camel=\"isOpened\""
        )]
    );
}

#[test]
fn truncated_alias_and_boundary_insertion_coalesce_after_utf16_crlf_expansion() {
    let source = "<script setup>const isOpened = true;</script>\r\n<template>😀<Child :is-opened /></template>";
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    let ctx = IdeContext::testing(&state, &uri, 0, source.into());
    let mut truncated = edit(source, "is-opened", "visible");
    truncated.range.end.character -= 1;
    let mut insertion = edit(source, "is-opened", "visible");
    insertion.range.start = insertion.range.end;
    let mut response = WorkspaceEdit {
        document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
            text_document: OptionalVersionedTextDocumentIdentifier {
                uri: uri.clone(),
                version: Some(7),
            },
            edits: vec![OneOf::Left(truncated), OneOf::Left(insertion)],
        }])),
        ..Default::default()
    };
    assert!(rewrite(
        &ctx,
        &document(&uri, source),
        &mut response,
        "visible",
        false
    ));
    assert!(coherent(&response));
    let Some(DocumentChanges::Edits(edits)) = response.document_changes else {
        panic!("document edits")
    };
    assert_eq!(edits[0].text_document.version, Some(7));
    assert_eq!(
        edits[0].edits,
        [OneOf::Left(edit(
            source,
            ":is-opened",
            ":is-opened=\"visible\""
        ))]
    );
}

#[test]
fn conflicting_plain_and_annotated_edits_refuse_the_whole_transaction() {
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    let source = "<script setup>const value = 1;</script>";
    let primary = edit(source, "value", "next");
    let conflict = edit(source, "value", "different");
    let response = WorkspaceEdit {
        changes: Some(HashMap::from([(uri.clone(), vec![primary])])),
        document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
            text_document: OptionalVersionedTextDocumentIdentifier {
                uri,
                version: Some(3),
            },
            edits: vec![OneOf::Right(AnnotatedTextEdit {
                text_edit: conflict,
                annotation_id: "review".into(),
            })],
        }])),
        ..Default::default()
    };
    assert!(!coherent(&response));
}

#[test]
fn reactive_destructure_cursor_chooses_the_local_binding_role() {
    let source = "<script setup lang=\"ts\">const { checked } = defineProps<{ checked: boolean }>();</script>";
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    let local = source.find("checked").unwrap();
    let ctx = IdeContext::testing(&state, &uri, local, source.into());
    assert!(is_binding_declaration(&ctx));
    let declaration = source.rfind("checked").unwrap();
    let ctx = IdeContext::testing(&state, &uri, declaration, source.into());
    assert!(!is_binding_declaration(&ctx));
}
