use vize_s0::cstr;

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
    assert_eq!(
        state.type_checker_vue_version(),
        vize_s0::config::VueVersion::V3
    );
    assert!(state.options_api_enabled());

    let logged_payload = cstr!("{features:?}");
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

    let logged_payload = cstr!("{features:?}");
    assert!(logged_payload.contains("legacy_vue2: false"));
    assert!(logged_payload.contains("options_api: false"));
}

#[test]
fn both_config_loaders_preserve_exact_template_globals() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{
          "globalTypes": {
            "currentRoute": {
              "type": "{ path: string }",
              "defaultValue": "{ path: '/' }"
            },
            "toThousandFilter": "(value: number) => string"
          }
        }"#,
    )
    .unwrap();

    for load_workspace in [true, false] {
        let state = ServerState::new();
        if load_workspace {
            state.load_workspace_config(dir.path());
        } else {
            state.load_lsp_config(dir.path());
        }

        let globals = state
            .virtual_ts_options()
            .template_globals
            .into_iter()
            .map(|global| (global.name, global.type_annotation, global.default_value))
            .collect::<Vec<_>>();
        assert_eq!(
            globals,
            vec![
                (
                    "currentRoute".into(),
                    "{ path: string }".into(),
                    "{ path: '/' }".into(),
                ),
                (
                    "toThousandFilter".into(),
                    "(value: number) => string".into(),
                    "{} as any".into(),
                ),
            ],
            "load_workspace={load_workspace}",
        );
    }
}

#[test]
fn vue_version_2_7_alone_enables_legacy_lowering() {
    // `vize check` honors `vue.version: "2.7"` without `typeChecker.legacyVue2`;
    // the LSP must derive legacy template lowering from the same key or it
    // publishes TS2304/TS2552 on pristine slot-scope/filter files that the CLI
    // accepts under the identical config (#3297).
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{ "vue": { "version": "2.7" } }"#,
    )
    .unwrap();

    let state = ServerState::new();
    state.load_workspace_config(dir.path());

    assert!(state.legacy_vue2_enabled());
    assert_eq!(
        state.type_checker_vue_version(),
        vize_s0::config::VueVersion::V2_7
    );
    assert!(
        state.options_api_enabled(),
        "legacy mode implies Options API"
    );
}

#[test]
fn compiler_compatibility_vue_version_2_alone_enables_legacy_lowering() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{ "compiler": { "compatibility": { "vueVersion": "2" } } }"#,
    )
    .unwrap();

    let state = ServerState::new();
    state.load_workspace_config(dir.path());

    assert!(state.legacy_vue2_enabled());
    assert_eq!(
        state.type_checker_vue_version(),
        vize_s0::config::VueVersion::V2
    );
}

#[test]
fn vue_version_3_keeps_legacy_lowering_disabled() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{ "vue": { "version": "3" } }"#,
    )
    .unwrap();

    let state = ServerState::new();
    state.load_workspace_config(dir.path());

    assert!(!state.legacy_vue2_enabled());
    assert_eq!(
        state.type_checker_vue_version(),
        vize_s0::config::VueVersion::V3
    );
}
