use tower_lsp::lsp_types::{Diagnostic, NumberOrString, Position, Range};

use super::{sources, without_duplicate_required_props};

#[test]
fn unavailable_notice_stays_in_the_carried_native_publication() {
    let state = crate::server::ServerState::new();
    let mut publication = super::combine(
        Vec::new(),
        false,
        Some(super::NativeFamily {
            diagnostics: Vec::new(),
            owns_script_syntax: false,
        }),
    );
    super::with_unavailable_hint(&state, &mut publication);
    let (family, wire) = publication.into_parts();
    assert_eq!(wire.len(), 1);
    assert_eq!(family.unwrap().diagnostics, wire);
}

#[test]
fn unsaved_required_prop_edit_reports_one_authored_diagnostic() {
    let Some(tsgo_path) = super::super::editor_typecheck_fixture::resolve_test_tsgo_binary() else {
        return;
    };
    let project = tempfile::tempdir().expect("temp project");
    let root = project.path();
    let src = root.join("src");
    std::fs::create_dir_all(&src).expect("src");
    super::super::editor_typecheck_fixture::write_vue_test_package(root);
    std::fs::write(
        root.join("tsconfig.json"),
        r#"{"compilerOptions":{"strict":true,"module":"ESNext","moduleResolution":"bundler","target":"ESNext","noEmit":true},"include":["src/**/*"]}"#,
    )
    .expect("tsconfig");
    super::super::editor_typecheck_fixture::write_corsa_config(root, &tsgo_path);
    let child = "<script setup lang='ts'>defineProps<{ label: string }>()</script>";
    let parent = "<script setup lang='ts'>import Child from './Child.vue'</script><template><Child label='ok' /></template>";
    let child_path = src.join("Child.vue");
    let parent_path = src.join("Parent.vue");
    std::fs::write(&child_path, child).expect("child");
    std::fs::write(&parent_path, parent).expect("parent");
    let child_uri = tower_lsp::lsp_types::Url::from_file_path(child_path).expect("child URI");
    let parent_uri = tower_lsp::lsp_types::Url::from_file_path(parent_path).expect("parent URI");
    let state =
        super::super::editor_typecheck_fixture::state_for_fixture(root, &parent_uri, parent);
    state.load_workspace_config(root);
    state
        .documents
        .open(child_uri.clone(), child.into(), 1, "vue".into());
    state.update_virtual_docs(&parent_uri, parent);
    state.update_virtual_docs(&child_uri, child);

    let initial = crate::runtime::block_on(super::super::DiagnosticService::collect_async(
        &state,
        &parent_uri,
    ));
    assert!(
        initial
            .iter()
            .all(|diagnostic| diagnostic.source.as_deref() != Some(sources::COMPONENTS)),
        "the initial child contract is satisfied: {initial:#?}",
    );

    let edited = child.replace("label: string", "label: string; extra: string");
    assert!(state.documents.apply_changes(
        &child_uri,
        vec![tower_lsp::lsp_types::TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: edited.clone(),
        }],
        2,
    ));
    state.update_virtual_docs(&child_uri, &edited);
    let diagnostics = crate::runtime::block_on(super::super::DiagnosticService::collect_async(
        &state,
        &parent_uri,
    ));
    let authored = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.source.as_deref() == Some(sources::COMPONENTS))
        .collect::<Vec<_>>();
    assert_eq!(
        authored.len(),
        1,
        "expected one missing prop: {diagnostics:#?}"
    );
    assert!(authored[0].message.contains("`extra`"));
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.message.contains("$ComponentCheckProps")),
        "generated checker types must not leak into duplicate editor diagnostics: {diagnostics:#?}",
    );
}

#[test]
fn required_prop_dedup_preserves_other_type_errors() {
    let tag = Range::new(Position::new(5, 3), Position::new(5, 8));
    let required = Diagnostic {
        range: tag,
        source: Some(sources::COMPONENTS.into()),
        code: Some(NumberOrString::String("component-required-props".into())),
        message: "<Child> is missing required prop: `extra`".into(),
        ..Default::default()
    };
    let duplicate = Diagnostic {
        range: Range::new(Position::new(5, 2), Position::new(5, 8)),
        source: Some(sources::TYPE_CHECKER.into()),
        code: Some(NumberOrString::Number(2345)),
        message: "Argument of type '{}' is not assignable to $ComponentCheckProps. Property 'extra' is missing".into(),
        ..Default::default()
    };
    let real_type_error = Diagnostic {
        range: Range::new(Position::new(5, 16), Position::new(5, 21)),
        message: "Argument of type 'number' is not assignable to type 'string'".into(),
        ..duplicate.clone()
    };
    let elsewhere = Diagnostic {
        range: Range::new(Position::new(6, 2), Position::new(6, 8)),
        ..duplicate.clone()
    };
    assert_eq!(
        without_duplicate_required_props(
            vec![
                duplicate.clone().into(),
                real_type_error.clone().into(),
                elsewhere.clone().into()
            ],
            std::slice::from_ref(&required),
        ),
        vec![real_type_error, elsewhere],
    );
    assert_eq!(
        without_duplicate_required_props(vec![duplicate.clone().into()], &[]),
        vec![duplicate],
    );
}

/// #11626: the prompt set shows a fresh script parser error beside a carried
/// type error; a family for the current text still replaces parser errors.
#[test]
fn carried_family_keeps_fresh_script_parser_errors() {
    let parser = Diagnostic {
        range: Range::new(Position::new(1, 6), Position::new(1, 7)),
        source: Some(sources::SCRIPT_PARSER.into()),
        severity: Some(tower_lsp::lsp_types::DiagnosticSeverity::ERROR),
        message: "Expected identifier".into(),
        ..Default::default()
    };
    let typed = Diagnostic {
        range: Range::new(Position::new(0, 6), Position::new(0, 7)),
        source: Some(sources::TYPE_CHECKER.into()),
        code: Some(NumberOrString::Number(2322)),
        message: "Type 'string' is not assignable to type 'number'.".into(),
        ..Default::default()
    };
    let carried = super::NativeFamily {
        diagnostics: vec![typed.clone().into()],
        owns_script_syntax: false,
    };
    assert_eq!(
        super::combine(vec![parser.clone()], true, Some(carried)).diagnostics,
        vec![parser.clone(), typed.clone()]
    );
    let current = super::NativeFamily {
        diagnostics: vec![typed.clone().into()],
        owns_script_syntax: true,
    };
    assert_eq!(
        super::combine(vec![parser], true, Some(current)).diagnostics,
        vec![typed]
    );
}
