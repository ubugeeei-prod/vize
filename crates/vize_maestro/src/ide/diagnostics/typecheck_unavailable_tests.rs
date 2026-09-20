use super::{
    TYPECHECK_UNAVAILABLE_HINT_MESSAGE, TYPECHECK_UNAVAILABLE_NOTICE_MESSAGE,
    service::typecheck_unavailable_hint, sources,
};
use tower_lsp::lsp_types::{DiagnosticSeverity, NumberOrString};

#[test]
fn unavailable_typecheck_diagnostic_points_to_typescript_7() {
    let hint = typecheck_unavailable_hint();

    assert_eq!(hint.source.as_deref(), Some(sources::TYPE_CHECKER));
    assert_eq!(hint.severity, Some(DiagnosticSeverity::HINT));
    assert_eq!(
        hint.code,
        Some(NumberOrString::String("typecheck-unavailable".to_string()))
    );
    assert_eq!(hint.message, TYPECHECK_UNAVAILABLE_HINT_MESSAGE);
    assert_ts7_guidance(&hint.message);
}

#[test]
fn unavailable_typecheck_notice_points_to_typescript_7() {
    assert_ts7_guidance(TYPECHECK_UNAVAILABLE_NOTICE_MESSAGE);
}

#[test]
fn a_failed_native_backend_preserves_the_script_parser_fallback() {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("vize.config.json"),
        r#"{"lsp":{"lint":false,"typecheck":true},"typeChecker":{"corsaPath":"./missing-syntax-checker"}}"#,
    ).unwrap();
    let state = crate::server::ServerState::new();
    state.load_lsp_config(project.path());
    state.set_workspace_root(project.path().to_path_buf());
    let uri = tower_lsp::lsp_types::Url::from_file_path(project.path().join("Broken.vue")).unwrap();
    state.documents.open(
        uri.clone(),
        "<script setup lang=\"ts\">\nconst label = ;\n</script><template />".into(),
        1,
        "vue".into(),
    );
    let diagnostics =
        crate::runtime::block_on(super::DiagnosticService::collect_async(&state, &uri));
    assert_eq!(
        diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.clone())
            .collect::<Vec<_>>(),
        [
            Some(NumberOrString::String("script-parse-error".into())),
            Some(NumberOrString::String("typecheck-unavailable".into())),
        ]
    );
    assert_eq!(
        diagnostics[0].range.start,
        tower_lsp::lsp_types::Position::new(1, 14)
    );
}

fn assert_ts7_guidance(message: &str) {
    assert!(message.contains("typescript@^7"));
    assert!(message.contains("typeChecker.corsaPath"));
    assert!(!message.contains("@typescript/native-preview"));
}
