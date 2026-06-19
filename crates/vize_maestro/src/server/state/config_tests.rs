use super::ServerState;

#[test]
fn language_server_legacy_vue2_reaches_logged_lsp_feature_payload() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{
          "languageServer": {
            "enabled": true,
            "lint": true,
            "typecheck": true,
            "editor": true,
            "legacyVue2": true,
            "formatting": true
          },
          "typeChecker": {
            "optionsApi": true,
            "tsconfig": "tsconfig.vize.json"
          }
        }"#,
    )
    .unwrap();

    let state = ServerState::new();
    state.load_workspace_config(dir.path());

    let features = state.lsp_features();
    assert!(
        features.legacy_vue2,
        "languageServer.legacyVue2 must be stored in runtime LSP features"
    );
    assert!(
        features.options_api,
        "typeChecker.optionsApi should be reflected in logged runtime LSP features"
    );
    assert!(state.legacy_vue2_enabled());
    assert!(state.options_api_enabled());

    let logged_payload = format!("{features:?}");
    assert!(logged_payload.contains("legacy_vue2: true"));
    assert!(logged_payload.contains("options_api: true"));
}

#[test]
fn disabled_language_server_clears_logged_legacy_vue2_feature() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{
          "languageServer": {
            "enabled": false,
            "legacyVue2": true
          }
        }"#,
    )
    .unwrap();

    let state = ServerState::new();
    state.load_workspace_config(dir.path());

    let features = state.lsp_features();
    assert!(
        !features.legacy_vue2,
        "languageServer.enabled: false should disable LSP compatibility flags too"
    );
    assert!(
        !features.options_api,
        "disabled LSP feature config should not log implied Options API support"
    );

    let logged_payload = format!("{features:?}");
    assert!(logged_payload.contains("legacy_vue2: false"));
    assert!(logged_payload.contains("options_api: false"));
}
