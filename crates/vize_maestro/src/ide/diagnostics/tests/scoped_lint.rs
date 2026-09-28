use super::{DiagnosticService, NumberOrString, Url, sources, state_with_lsp_diagnostics};

#[test]
fn diagnostics_follow_scoped_rules_and_global_ignores() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("vize.config.json"),
        r#"{"entries":[{"files":["src/legacy/**/*.vue"],"linter":{"rules":{"vue/permitted-contents":"off"}}}],"ignores":["src/generated/**"]}"#,
    )
    .unwrap();
    let state = state_with_lsp_diagnostics(true, false);
    state.apply_initialize_workspace_folders(None, Some(root.path()));
    let source =
        "<template><button type=\"button\"><div class=\"content\">1</div></button></template>";
    for (path, should_lint) in [
        ("src/legacy/table.vue", false),
        ("src/other.vue", true),
        ("src/generated/table.vue", false),
    ] {
        let uri = Url::from_file_path(root.path().join(path)).unwrap();
        state
            .documents
            .open(uri.clone(), source.into(), 1, "vue".into());
        let diagnostics = DiagnosticService::collect(&state, &uri);
        let has_rule = diagnostics.iter().any(|diagnostic| {
            diagnostic.source.as_deref() == Some(sources::LINTER)
                && diagnostic.code == Some(NumberOrString::String("vue/permitted-contents".into()))
        });
        assert_eq!(has_rule, should_lint, "{path}: {diagnostics:?}");
    }
}
