use super::DiagnosticService;
use crate::server::ServerState;
use tower_lsp::lsp_types::{DiagnosticSeverity, NumberOrString, Url};

#[test]
fn configured_globals_reach_the_shared_editor_linter() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("vize.config.json"),
        include_str!(
            "../../../../vize_patina/tests/fixtures/global-component-registration/vize.config.json"
        ),
    )
    .unwrap();
    let state = ServerState::new();
    state.load_lsp_config(root.path());
    let uri = Url::from_file_path(root.path().join("Card.vue")).unwrap();
    state.documents.open(
        uri.clone(),
        "<template><MyButton /><my-button /><MissingWidget /></template>".into(),
        1,
        "vue".into(),
    );
    let diagnostics = DiagnosticService::collect_lint_only(&state, &uri);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code,
        Some(NumberOrString::String(
            "vue/require-component-registration".into()
        ))
    );
    assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::ERROR));
    assert_eq!(diagnostics[0].range.start.character, 36);
}
