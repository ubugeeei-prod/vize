use super::{
    load_compiler_template_syntax, load_config_and_linter_with_source, load_config_with_source,
    load_linter_config, validate_explicit_config_path,
};

#[test]
fn validate_explicit_config_path_missing_errors() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("does-not-exist.toml");

    let result = validate_explicit_config_path(&missing);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("config file not found"));
}

#[test]
fn validate_explicit_config_path_malformed_errors() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.json");
    std::fs::write(&config_path, "this is { not valid json ===").unwrap();

    let result = validate_explicit_config_path(&config_path);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("failed to parse"));
}

#[test]
fn validate_explicit_config_path_valid_ok() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.json");
    std::fs::write(&config_path, r#"{ "formatter": { "singleQuote": true } }"#).unwrap();

    assert!(validate_explicit_config_path(&config_path).is_ok());
}

#[test]
fn validate_explicit_config_path_dir_with_config_ok() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{ "formatter": {} }"#,
    )
    .unwrap();

    assert!(validate_explicit_config_path(dir.path()).is_ok());
}

#[test]
fn validate_explicit_config_path_empty_dir_errors() {
    let dir = tempfile::tempdir().unwrap();
    let result = validate_explicit_config_path(dir.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("no vize config file found"));
}

#[test]
fn load_config_uses_explicit_file_path() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("custom.json");
    std::fs::write(&config_path, r#"{ "formatter": { "singleQuote": true } }"#).unwrap();

    let loaded = load_config_with_source(Some(&config_path));
    assert_eq!(loaded.source_path.as_deref(), Some(config_path.as_path()));
    assert!(loaded.config.formatter.single_quote);
}

#[test]
fn load_config_reads_dialect_key() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.json");
    std::fs::write(&config_path, r#"{ "dialect": "petite-vue" }"#).unwrap();

    let loaded = load_config_with_source(Some(&config_path));
    assert_eq!(
        loaded.config.dialect,
        Some(crate::dialect::VueDialect::PetiteVue)
    );
}

#[test]
fn load_config_defaults_dialect_to_unset() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.json");
    std::fs::write(&config_path, r#"{ "formatter": { "singleQuote": true } }"#).unwrap();

    let loaded = load_config_with_source(Some(&config_path));
    assert_eq!(loaded.config.dialect, None);
}

#[test]
fn load_config_reads_compiler_template_syntax() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.json");
    std::fs::write(
        &config_path,
        r#"{ "compiler": { "templateSyntax": "quirks" } }"#,
    )
    .unwrap();

    assert_eq!(
        load_compiler_template_syntax(Some(&config_path)),
        Some("quirks")
    );
}

#[test]
fn load_config_uses_typescript_config() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.ts");
    std::fs::write(
        &config_path,
        r#"
export default {
  linter: {
    rules: {
      "vue/prop-name-casing": "off",
      "script/no-options-api": "error",
    },
  },
}
"#,
    )
    .unwrap();

    let (loaded, linter_from_loaded_config) = load_config_and_linter_with_source(Some(dir.path()));
    let linter = load_linter_config(Some(dir.path()));

    assert_eq!(loaded.source_path.as_deref(), Some(config_path.as_path()));
    assert_eq!(
        linter_from_loaded_config.disabled_rules(),
        ["vue/prop-name-casing"]
    );
    assert_eq!(
        linter_from_loaded_config.enabled_rules(),
        ["script/no-options-api"]
    );
    assert_eq!(linter.disabled_rules(), ["vue/prop-name-casing"]);
    assert_eq!(linter.enabled_rules(), ["script/no-options-api"]);
}
