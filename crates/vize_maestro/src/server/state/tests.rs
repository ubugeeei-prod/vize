use super::ServerState;
use crate::virtual_code::{ArtCursorPosition, BlockType, find_art_block_at_offset};
use tower_lsp::lsp_types::Url;

#[cfg(feature = "native")]
#[test]
fn corsa_init_failure_is_recorded() {
    let state = ServerState::new();
    assert!(state.corsa_init_failure().is_none());
    state.record_corsa_init_failure("spawn failed: missing tsgo");
    let reason = state
        .corsa_init_failure()
        .expect("failure reason should be recorded");
    assert!(
        reason.contains("spawn failed"),
        "reason should preserve the recorded message: {reason}"
    );
}

#[test]
fn default_format_options() {
    let state = ServerState::new();
    let opts = state.get_format_options();
    assert_eq!(opts.print_width, 100);
    assert_eq!(opts.tab_width, 2);
    assert!(!opts.use_tabs);
    assert!(opts.semi);
    assert!(!opts.single_quote);
    assert!(opts.sort_attributes);
    assert!(opts.normalize_directive_shorthands);
}

#[test]
fn load_format_config_no_file() {
    let dir = tempfile::tempdir().unwrap();
    let state = ServerState::new();
    state.load_format_config(dir.path());
    // options remain default
    let opts = state.get_format_options();
    assert_eq!(opts.print_width, 100);
}

#[test]
fn load_format_config_from_file() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{
                "fmt": {
                    "printWidth": 80,
                    "tabWidth": 4,
                    "useTabs": true,
                    "semi": false,
                    "singleQuote": true
                }
            }"#,
    )
    .unwrap();

    let state = ServerState::new();
    state.load_format_config(dir.path());
    let opts = state.get_format_options();
    assert_eq!(opts.print_width, 80);
    assert_eq!(opts.tab_width, 4);
    assert!(opts.use_tabs);
    assert!(!opts.semi);
    assert!(opts.single_quote);
}

#[test]
fn load_format_config_partial() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{ "fmt": { "printWidth": 120 } }"#,
    )
    .unwrap();

    let state = ServerState::new();
    state.load_format_config(dir.path());
    let opts = state.get_format_options();
    assert_eq!(opts.print_width, 120);
    // defaults preserved
    assert_eq!(opts.tab_width, 2);
    assert!(opts.semi);
}

#[test]
fn load_format_config_no_fmt_section() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{ "check": { "globals": ["$t"] } }"#,
    )
    .unwrap();

    let state = ServerState::new();
    state.load_format_config(dir.path());
    // no fmt section → options remain default
    let opts = state.get_format_options();
    assert_eq!(opts.print_width, 100);
}

#[test]
fn load_format_config_invalid_json() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("vize.config.json"), "not valid json").unwrap();

    let state = ServerState::new();
    state.load_format_config(dir.path());
    // options remain default
    let opts = state.get_format_options();
    assert_eq!(opts.print_width, 100);
}

#[test]
fn lsp_features_enable_non_opinionated_defaults() {
    let state = ServerState::new();
    let features = state.lsp_features();
    assert!(features.lint);
    assert!(features.typecheck);
    assert!(features.ecosystem);
    assert!(features.completion);
    assert!(features.signature_help);
    assert!(features.code_actions);
    assert!(!features.formatting);
    assert!(state.is_lsp_lint_enabled());
    assert!(state.is_lsp_typecheck_enabled());
}

#[test]
fn load_lsp_config_from_json() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{
                "lsp": {
                    "lint": true,
                    "typecheck": true,
                    "editor": true,
                    "signatureHelp": false,
                    "formatting": false
                }
            }"#,
    )
    .unwrap();

    let state = ServerState::new();
    state.load_lsp_config(dir.path());
    let features = state.lsp_features();
    assert!(features.lint);
    assert!(features.typecheck);
    assert!(features.ecosystem);
    assert!(features.completion);
    assert!(!features.signature_help);
    assert!(features.definition);
    assert!(!features.formatting);
}

#[test]
fn load_lsp_config_updates_type_checker_config() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{
                "typeChecker": {
                    "strict": true,
                    "checkProps": false,
                    "checkEmits": false,
                    "tsconfig": "tsconfig.app.json",
                    "corsaPath": "./node_modules/.bin/corsa"
                }
            }"#,
    )
    .unwrap();

    let state = ServerState::new();
    state.load_lsp_config(dir.path());
    let config = state.get_type_checker_config();
    assert!(config.strict);
    assert!(!config.check_props);
    assert!(!config.check_emits);
    assert_eq!(config.tsconfig.as_deref(), Some("tsconfig.app.json"));
    assert_eq!(config.runtime_path(), Some("./node_modules/.bin/corsa"));
}

#[test]
fn options_api_enabled_by_default() {
    let state = ServerState::new();
    assert!(
        state.options_api_enabled(),
        "Options API resolution is default-on (matches vue-tsc); template bindings \
         resolve without configuration"
    );
}

#[test]
fn type_checker_options_api_opt_out_from_config() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{ "typeChecker": { "optionsApi": false } }"#,
    )
    .unwrap();

    let state = ServerState::new();
    state.load_workspace_config(dir.path());
    assert!(
        !state.options_api_enabled(),
        "typeChecker.optionsApi: false should opt out of Options API binding resolution in the LSP"
    );
}

#[test]
fn type_checker_options_api_explicit_opt_in_from_config() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{ "typeChecker": { "optionsApi": true } }"#,
    )
    .unwrap();

    let state = ServerState::new();
    state.load_workspace_config(dir.path());
    assert!(
        state.options_api_enabled(),
        "typeChecker.optionsApi: true keeps Options API binding resolution enabled in the LSP"
    );
}

#[test]
fn legacy_vue2_config_implies_options_api() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{ "typeChecker": { "legacyVue2": true } }"#,
    )
    .unwrap();

    let state = ServerState::new();
    state.load_workspace_config(dir.path());
    assert!(
        state.legacy_vue2_enabled(),
        "typeChecker.legacyVue2 should enable Vue 2 compatibility"
    );
    assert!(
        state.options_api_enabled(),
        "legacy Vue 2 mode is a superset of Options API binding resolution"
    );
}

#[test]
fn jsx_typecheck_defaults_off() {
    let state = ServerState::new();
    assert!(
        !state.jsx_typecheck_enabled(),
        "JSX/TSX type-aware features must default off so React .tsx is untouched"
    );
}

#[test]
fn jsx_typecheck_config_opts_in() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{ "typeChecker": { "jsxTypecheck": true } }"#,
    )
    .unwrap();

    let state = ServerState::new();
    state.load_workspace_config(dir.path());
    assert!(
        state.jsx_typecheck_enabled(),
        "typeChecker.jsxTypecheck: true should enable JSX/TSX type-aware LSP"
    );
}

#[test]
fn load_lsp_config_updates_linter_config() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{
                "linter": {
                    "preset": "opinionated",
                    "rules": {
                        "vue/prop-name-casing": "off"
                    }
                }
            }"#,
    )
    .unwrap();

    let state = ServerState::new();
    state.load_lsp_config(dir.path());
    let config = state.get_linter_config();
    assert_eq!(config.preset.as_deref(), Some("opinionated"));
    assert_eq!(config.disabled_rules(), ["vue/prop-name-casing"]);
}

#[test]
fn load_lsp_config_invalid_json_keeps_default() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("vize.config.json"), "not valid json").unwrap();

    let state = ServerState::new();
    state.load_lsp_config(dir.path());
    assert_eq!(state.lsp_features(), super::LspFeatureConfig::default());
}

#[test]
fn apply_lsp_initialization_options() {
    let state = ServerState::new();
    let options = serde_json::json!({
        "lint": true,
        "codeActions": true,
        "definition": true,
        "ecosystem": true
    });

    state.apply_lsp_initialization_options(Some(&options));

    let features = state.lsp_features();
    assert!(features.lint);
    assert!(features.code_actions);
    assert!(features.definition);
    assert!(features.ecosystem);
    assert!(features.typecheck);
}

mod virtual_docs;
