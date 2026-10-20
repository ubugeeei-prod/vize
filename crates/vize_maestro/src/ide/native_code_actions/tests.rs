use super::*;
use crate::{ide::DiagnosticService, server::ServerState};
use std::collections::HashMap;
use tower_lsp::lsp_types::{Command, Url};

const REFACTOR_FIXTURE: &str =
    include_str!("../../../tests/fixtures/native-refactor-expression.vue");

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
        source_catalogs: Vec::new(),
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
    let quickfix = [CodeActionKind::QUICKFIX];
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
    let mapped = map_action(
        &ctx,
        &document,
        7,
        &quickfix,
        make_action(vec![edit.clone()]),
    )
    .unwrap();
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
            &quickfix,
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
    assert!(map_action(&ctx, &document, 7, &quickfix, command).is_none());
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
    assert!(map_action(&ctx, &document, 7, &quickfix, foreign).is_none());
    assert!(
        map_action(
            &ctx,
            &document,
            7,
            &quickfix,
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
            &quickfix,
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

#[test]
fn refactor_edits_map_from_the_canonical_document_to_the_authored_fixture() {
    let uri = Url::parse("file:///workspace/Refactor.vue").unwrap();
    let state = ServerState::new();
    state.documents.open(
        uri.clone(),
        REFACTOR_FIXTURE.to_string(),
        11,
        "vue".to_string(),
    );
    let source_offset = REFACTOR_FIXTURE.find("base + 2").unwrap();
    let ctx = IdeContext::new(&state, &uri, source_offset).unwrap();
    let document = corsa_support::CanonicalVirtualDocument {
        source_uri: uri.clone(),
        request_uri: "file:///workspace/Refactor.vue.ts".into(),
        virtual_result: DiagnosticService::generate_virtual_ts(
            &uri,
            REFACTOR_FIXTURE,
            false,
            false,
        )
        .unwrap(),
        dependencies: vec![],
        materialized_sources: vec![],
        session_project_roots: vec![],
        source_catalogs: Vec::new(),
    };
    let generated = &document.virtual_result.code;
    let generated_offset = generated.find("base + 2").unwrap();
    let (line, character) = crate::ide::offset_to_position(generated, generated_offset);
    let generated_range = Range::new(
        Position::new(line, character),
        Position::new(line, character + "base + 2".len() as u32),
    );
    let kind = CodeActionKind::from("refactor.rewrite.expression".to_string());
    let action = CodeAction {
        title: "Rewrite expression".to_string(),
        kind: Some(kind.clone()),
        edit: Some(WorkspaceEdit {
            changes: Some(HashMap::from([(
                Url::parse(&document.request_uri).unwrap(),
                vec![TextEdit {
                    range: generated_range,
                    new_text: "2 + base".to_string(),
                }],
            )])),
            ..Default::default()
        }),
        ..Default::default()
    };

    let mapped = map_action(
        &ctx,
        &document,
        11,
        std::slice::from_ref(&CodeActionKind::REFACTOR),
        action.clone(),
    )
    .expect("parent refactor request accepts its authored descendant");
    assert_eq!(mapped.kind.as_ref(), Some(&kind));
    let Some(DocumentChanges::Edits(changes)) = mapped.edit.unwrap().document_changes else {
        panic!("expected versioned authored refactor edits");
    };
    assert_eq!(changes[0].text_document.uri, uri);
    assert_eq!(changes[0].text_document.version, Some(11));
    let OneOf::Left(edit) = &changes[0].edits[0] else {
        panic!("annotated refactor edits are not forwarded");
    };
    let (start_line, start_character) =
        crate::ide::offset_to_position(REFACTOR_FIXTURE, source_offset);
    let (end_line, end_character) =
        crate::ide::offset_to_position(REFACTOR_FIXTURE, source_offset + "base + 2".len());
    assert_eq!(
        edit.range,
        Range::new(
            Position::new(start_line, start_character),
            Position::new(end_line, end_character),
        )
    );
    assert!(
        map_action(
            &ctx,
            &document,
            11,
            std::slice::from_ref(&CodeActionKind::REFACTOR_EXTRACT),
            action,
        )
        .is_none(),
        "a sibling refactor kind must not escape an explicit only request"
    );
}

#[test]
fn inserted_lines_take_the_authored_indent_and_line_ending() {
    let member = "\n      area(): number {\n          return 1;\n      }";
    assert_eq!(
        relayout(member, "    ", ("  ", false)),
        "\n    area(): number {\n        return 1;\n    }"
    );
    assert_eq!(
        relayout(member, "    ", ("  ", true)),
        "\r\n    area(): number {\r\n        return 1;\r\n    }"
    );
    assert_eq!(relayout("\n", "  ", ("", true)), "\r\n");
    assert_eq!(relayout("single line", "  ", ("", true)), "single line");
    // A line outside the generated indent keeps the whole text as tsgo sent it.
    let literal = "\n  a = `\nraw`;";
    assert_eq!(relayout(literal, "  ", ("", false)), literal);
}
