use tower_lsp::lsp_types::{Diagnostic, NumberOrString, Position, Range};

use super::{sources, without_duplicate_required_props};

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
            .all(|diagnostic| !diagnostic.message.contains("__VizeComponentCheckProps")),
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
        message: "Argument of type '{}' is not assignable to __VizeComponentCheckProps. Property 'extra' is missing".into(),
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
                duplicate.clone(),
                real_type_error.clone(),
                elsewhere.clone()
            ],
            std::slice::from_ref(&required),
        ),
        vec![real_type_error, elsewhere],
    );
    assert_eq!(
        without_duplicate_required_props(vec![duplicate.clone()], &[]),
        vec![duplicate],
    );
}
