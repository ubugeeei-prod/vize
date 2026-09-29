use vize_carton::config::{TypeCheckerConfig, load_lsp_request_timeout_ms};

#[test]
fn external_type_checker_struct_literal_keeps_the_published_fields() {
    let config = TypeCheckerConfig {
        enabled: true,
        strict: false,
        check_props: true,
        check_emits: true,
        check_template_bindings: true,
        check_reactivity: true,
        check_setup_context: true,
        check_invalid_exports: true,
        check_fallthrough_attrs: true,
        tsconfig: None,
        tsgo_path: None,
        globals_file: None,
        servers: None,
    };
    assert_eq!(config, TypeCheckerConfig::default());
}

#[test]
fn editor_timeout_loads_from_json_without_changing_the_shared_struct() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("vize.config.json");
    assert_eq!(load_lsp_request_timeout_ms(Some(dir.path())), 60_000);

    std::fs::write(&path, r#"{"typeChecker":{"lspRequestTimeoutMs":90000}}"#).unwrap();
    assert_eq!(load_lsp_request_timeout_ms(Some(dir.path())), 90_000);

    std::fs::write(&path, r#"{"typeChecker":{"lspRequestTimeoutMs":0}}"#).unwrap();
    assert_eq!(load_lsp_request_timeout_ms(Some(dir.path())), 1);

    std::fs::write(&path, r#"{"typeChecker":{}}"#).unwrap();
    assert_eq!(load_lsp_request_timeout_ms(Some(dir.path())), 60_000);
}
