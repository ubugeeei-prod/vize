use vize_carton::config::{TypeCheckerConfig, load_lsp_config_snapshot};

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
    assert_eq!(
        load_lsp_config_snapshot(Some(dir.path())).request_timeout_ms,
        60_000
    );

    std::fs::write(&path, r#"{"typeChecker":{"lspRequestTimeoutMs":90000}}"#).unwrap();
    assert_eq!(
        load_lsp_config_snapshot(Some(dir.path())).request_timeout_ms,
        90_000
    );

    std::fs::write(&path, r#"{"typeChecker":{"lspRequestTimeoutMs":0}}"#).unwrap();
    assert_eq!(
        load_lsp_config_snapshot(Some(dir.path())).request_timeout_ms,
        1
    );

    std::fs::write(&path, r#"{"typeChecker":{}}"#).unwrap();
    assert_eq!(
        load_lsp_config_snapshot(Some(dir.path())).request_timeout_ms,
        60_000
    );
}

#[test]
fn editor_snapshot_evaluates_function_config_once() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("eval-count.txt"), "0").unwrap();
    std::fs::write(
        dir.path().join("vize.config.mjs"),
        r#"
import { readFileSync, writeFileSync } from 'node:fs';
const counter = new URL('./eval-count.txt', import.meta.url);
export default () => {
  const count = Number(readFileSync(counter, 'utf8')) + 1;
  writeFileSync(counter, String(count));
  return { typeChecker: { strict: count === 1, lspRequestTimeoutMs: count === 1 ? 90000 : 120000 } };
};
"#,
    )
    .unwrap();

    let snapshot = load_lsp_config_snapshot(Some(dir.path()));
    assert_eq!(snapshot.request_timeout_ms, 90_000);
    assert!(snapshot.config.type_checker.strict);
    assert_eq!(
        std::fs::read_to_string(dir.path().join("eval-count.txt")).unwrap(),
        "1"
    );
}
