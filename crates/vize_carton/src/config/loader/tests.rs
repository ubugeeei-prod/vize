use super::{
    load_compiler_host_compiler, load_compiler_jsx_mode, load_compiler_template_syntax,
    load_compiler_vue_version, load_config_and_linter_with_source,
    load_config_entry_files_with_source, load_config_entry_ignores_with_source,
    load_config_with_source, load_linter_config, validate_explicit_config_path,
};
use crate::config::{JsxMode, VueVersion};

#[test]
fn validate_explicit_config_path_missing_errors() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("does-not-exist.toml");

    let result = validate_explicit_config_path(&missing);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        format!("config file not found: {}", missing.display())
    );
}

#[test]
fn validate_explicit_config_path_malformed_errors() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.json");
    std::fs::write(&config_path, "this is { not valid json ===").unwrap();

    let result = validate_explicit_config_path(&config_path);
    assert!(result.is_err());
    let error = result
        .unwrap_err()
        .replace(config_path.to_string_lossy().as_ref(), "<CONFIG>");
    insta::assert_snapshot!(error);
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
    assert_eq!(
        result.unwrap_err(),
        format!("no vize config file found under {}", dir.path().display())
    );
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
fn load_compiler_vue_version_reads_vue_version_key() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.json");
    std::fs::write(&config_path, r#"{ "vue": { "version": "2" } }"#).unwrap();

    assert_eq!(
        load_compiler_vue_version(Some(&config_path)),
        Some(VueVersion::V2)
    );
}

#[test]
fn load_compiler_vue_version_reads_compiler_compatibility_key() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.json");
    std::fs::write(
        &config_path,
        r#"{ "compiler": { "compatibility": { "vueVersion": "2.7" } } }"#,
    )
    .unwrap();

    assert_eq!(
        load_compiler_vue_version(Some(&config_path)),
        Some(VueVersion::V2_7)
    );
}

#[test]
fn load_compiler_host_compiler_reads_compiler_compatibility_key() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.json");
    std::fs::write(
        &config_path,
        r#"{ "compiler": { "compatibility": { "hostCompiler": false } } }"#,
    )
    .unwrap();

    assert_eq!(load_compiler_host_compiler(Some(&config_path)), Some(false));
}

#[test]
fn load_compiler_vue_version_defaults_to_unset_for_vue3() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.json");
    std::fs::write(&config_path, r#"{ "formatter": { "singleQuote": true } }"#).unwrap();

    // No `vue.version` key → modern Vue 3 (the absent/default dialect).
    assert_eq!(load_compiler_vue_version(Some(&config_path)), None);
}

#[test]
fn load_compiler_jsx_mode_reads_jsx_mode_key() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.json");
    std::fs::write(&config_path, r#"{ "compiler": { "jsxMode": "vapor" } }"#).unwrap();

    assert_eq!(
        load_compiler_jsx_mode(Some(&config_path)),
        Some(JsxMode::Vapor)
    );
    assert_eq!(
        load_compiler_jsx_mode(Some(&config_path)).map(JsxMode::as_str),
        Some("vapor")
    );
}

#[test]
fn load_compiler_jsx_mode_reads_vdom() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.json");
    std::fs::write(&config_path, r#"{ "compiler": { "jsxMode": "vdom" } }"#).unwrap();

    assert_eq!(
        load_compiler_jsx_mode(Some(&config_path)),
        Some(JsxMode::Vdom)
    );
}

#[test]
fn load_compiler_jsx_mode_defaults_to_unset() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.json");
    std::fs::write(&config_path, r#"{ "formatter": { "singleQuote": true } }"#).unwrap();

    // No `compiler.jsxMode` key → absent (the JSX entry points treat this as VDOM).
    assert_eq!(load_compiler_jsx_mode(Some(&config_path)), None);
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
    categories: {
      "style": "off",
      "a11y": "warn",
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
    assert_eq!(linter.disabled_categories(), ["style"]);
    assert_eq!(
        linter.category_severity_overrides(),
        [("a11y".into(), crate::config::LintRuleSeverity::Warn)]
    );
}

#[test]
fn load_linter_config_uses_common_entry_preset() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.json");
    std::fs::write(
        &config_path,
        r#"{
  "entries": [
    { "name": "app", "files": ["components/**/*.vue"], "linter": { "preset": "incremental" } },
    { "name": "design-system", "basePath": "design-system", "files": ["src/**/*.vue"], "linter": { "preset": "incremental" } }
  ]
}"#,
    )
    .unwrap();

    let (_, loaded_linter) = load_config_and_linter_with_source(Some(&config_path));
    let linter = load_linter_config(Some(&config_path));

    assert_eq!(loaded_linter.preset.as_deref(), Some("incremental"));
    assert_eq!(linter.preset.as_deref(), Some("incremental"));
}

#[test]
fn load_linter_config_keeps_root_preset_over_entry_preset() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.json");
    std::fs::write(
        &config_path,
        r#"{
  "linter": { "preset": "nuxt" },
  "entries": [
    { "files": ["components/**/*.vue"], "linter": { "preset": "incremental" } }
  ]
}"#,
    )
    .unwrap();

    let linter = load_linter_config(Some(&config_path));

    assert_eq!(linter.preset.as_deref(), Some("nuxt"));
}

#[test]
fn load_config_entry_ignores_preserves_base_paths() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.json");
    std::fs::write(
        &config_path,
        r#"{
  "ignores": ["src/generated.ts"],
  "entries": [
    { "name": "app", "ignores": ["components/Legacy.vue"] },
    { "name": "design", "basePath": "design-system", "ignores": ["src/Fixture.vue"] }
  ]
}"#,
    )
    .unwrap();

    let loaded = load_config_entry_ignores_with_source(Some(&config_path));

    assert_eq!(loaded.source_path.as_deref(), Some(config_path.as_path()));
    assert_eq!(loaded.ignores.len(), 3);
    assert_eq!(loaded.ignores[0].base_path, None);
    assert_eq!(loaded.ignores[0].pattern.as_str(), "src/generated.ts");
    assert_eq!(loaded.ignores[1].base_path, None);
    assert_eq!(loaded.ignores[1].pattern.as_str(), "components/Legacy.vue");
    assert_eq!(
        loaded.ignores[2].base_path.as_deref(),
        Some("design-system")
    );
    assert_eq!(loaded.ignores[2].pattern.as_str(), "src/Fixture.vue");
}

#[test]
fn load_config_entry_files_preserves_base_paths() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.json");
    std::fs::write(
        &config_path,
        r#"{
  "basePath": "app",
  "files": ["src/**/*.vue"],
  "entries": [
    { "name": "design", "basePath": "design-system", "files": ["src/**/*.art.vue"] }
  ]
}"#,
    )
    .unwrap();

    let loaded = load_config_entry_files_with_source(Some(&config_path));

    assert_eq!(loaded.source_path.as_deref(), Some(config_path.as_path()));
    assert_eq!(loaded.entries.len(), 2);
    assert_eq!(loaded.entries[0].base_path.as_deref(), Some("app"));
    assert_eq!(loaded.entries[0].files, vec!["src/**/*.vue"]);
    assert_eq!(
        loaded.entries[1].base_path.as_deref(),
        Some("design-system")
    );
    assert_eq!(loaded.entries[1].files, vec!["src/**/*.art.vue"]);
}

#[test]
fn load_config_uses_relative_typescript_file_path() {
    let cwd = std::env::current_dir().unwrap();
    let case_dir = cwd
        .join("target")
        .join(format!("vize-config-relative-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&case_dir);
    std::fs::create_dir_all(&case_dir).unwrap();
    let config_path = case_dir.join("vize.config.ts");
    std::fs::write(
        &config_path,
        r#"
export default {
  formatter: {
    singleQuote: true,
  },
}
"#,
    )
    .unwrap();
    let relative_config_path = config_path.strip_prefix(&cwd).unwrap();

    let loaded = load_config_with_source(Some(relative_config_path));

    assert!(loaded.config.formatter.single_quote);

    let _ = std::fs::remove_dir_all(&case_dir);
}
