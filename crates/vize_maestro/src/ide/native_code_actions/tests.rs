use super::*;
use crate::{ide::DiagnosticService, server::ServerState};
use std::collections::HashMap;
use tower_lsp::lsp_types::{Command, Url};

#[test]
fn quick_fixes_are_atomic_versioned_and_do_not_leak_backend_commands() {
    let source = "<script setup lang=\"ts\">const value = 1;</script>";
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.to_string(), 7, "vue".to_string());
    let ctx = IdeContext::new(&state, &uri, source.find("value").unwrap()).unwrap();
    let mut document = corsa_support::CanonicalVirtualDocument {
        source_uri: uri.clone(),
        request_uri: "file:///workspace/App.vue.ts".into(),
        virtual_result: DiagnosticService::generate_virtual_ts(&uri, source, false, false).unwrap(),
        dependencies: vec![],
        materialized_sources: vec![],
        session_project_roots: vec![],
    };
    document
        .virtual_result
        .code
        .push_str("\nconst compilerTemporary = 1;\n");
    let generated = &document.virtual_result.code;
    let offset = generated.find("value = 1").unwrap();
    let (line, character) = crate::ide::offset_to_position(generated, offset);
    let range = Range::new(
        Position::new(line, character),
        Position::new(line, character + 5),
    );
    let edit = TextEdit {
        range,
        new_text: "renamed".to_string(),
    };
    let make_action = |edits| CodeAction {
        title: "Rename binding".to_string(),
        kind: Some(CodeActionKind::QUICKFIX),
        edit: Some(WorkspaceEdit {
            changes: Some(HashMap::from([(
                Url::parse(&document.request_uri).unwrap(),
                edits,
            )])),
            ..Default::default()
        }),
        ..Default::default()
    };
    let mapped = map_action(&ctx, &document, 7, make_action(vec![edit.clone()])).unwrap();
    let Some(DocumentChanges::Edits(changes)) = mapped.edit.unwrap().document_changes else {
        panic!("expected versioned authored edits");
    };
    assert_eq!(changes[0].text_document.uri, uri);
    assert_eq!(changes[0].text_document.version, Some(7));
    assert_eq!(changes[0].edits.len(), 1);

    let synthetic = TextEdit {
        range: Range::new(Position::new(0, 0), Position::new(0, 0)),
        new_text: "unsafe".to_string(),
    };
    assert!(
        map_action(
            &ctx,
            &document,
            7,
            make_action(vec![edit.clone(), synthetic])
        )
        .is_none()
    );
    let mut command = make_action(vec![edit.clone()]);
    command.command = Some(Command {
        title: "apply".to_string(),
        command: "backend.apply".to_string(),
        arguments: None,
    });
    assert!(map_action(&ctx, &document, 7, command).is_none());
    let mut foreign = make_action(vec![edit.clone()]);
    foreign
        .edit
        .as_mut()
        .unwrap()
        .changes
        .as_mut()
        .unwrap()
        .insert(
            Url::parse("file:///workspace/Other.vue.ts").unwrap(),
            vec![edit.clone()],
        );
    assert!(map_action(&ctx, &document, 7, foreign).is_none());
    assert!(
        map_action(
            &ctx,
            &document,
            7,
            make_action(vec![TextEdit {
                new_text: "renamed = compilerTemporary; const other".to_string(),
                ..edit.clone()
            }])
        )
        .is_none()
    );
    assert!(
        map_action(
            &ctx,
            &document,
            7,
            make_action(vec![TextEdit {
                new_text: "__VizeAuthoredName".to_string(),
                ..edit
            }])
        )
        .is_some(),
        "authored identifiers are not reserved by prefix"
    );
}

#[test]
fn same_position_insertions_keep_order_and_overlapping_replacements_are_rejected() {
    let range = Range::new(Position::new(1, 3), Position::new(1, 3));
    let edit = |text: &str| TextEdit {
        range,
        new_text: text.to_string(),
    };
    assert_eq!(
        ordered_edits(vec![edit("first"), edit("second")]).unwrap(),
        vec![edit("firstsecond")]
    );
    let first = TextEdit {
        range: Range::new(Position::new(1, 1), Position::new(1, 5)),
        new_text: "a".to_string(),
    };
    assert!(ordered_edits(vec![first, edit("inside")]).is_none());
}
