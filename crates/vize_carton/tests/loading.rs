#![allow(clippy::disallowed_macros)]

use vize_carton::config::{
    VueVersion, load_config, load_config_entry_ignores_with_source,
    load_config_with_features_and_source, load_config_with_source, validate_explicit_config_path,
};

#[test]
fn loads_pkl_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.pkl");
    install_pkl_modules(dir.path());
    std::fs::write(
        &config_path,
        r#"
amends "node_modules/vize/pkl/VizeConfig.pkl"

formatter {
  singleQuote = true
}

languageServer {
  completion = false
}

typeChecker {
  globalsFile = "./globals.d.ts"
}
"#,
    )
    .unwrap();

    let config = load_config(Some(dir.path()));

    insta::assert_snapshot!(serde_json::to_string_pretty(&config).unwrap());
}

#[test]
fn loads_pkl_top_level_and_entry_ignores() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.pkl");
    install_pkl_modules(dir.path());
    std::fs::write(
        &config_path,
        r#"
amends "node_modules/vize/pkl/VizeConfig.pkl"

ignores = new Listing {
  "src/generated.ts"
}

entries = new Listing {
  new ConfigEntry {
    name = "app"
    ignores = new Listing { "components/Legacy.vue" }
  }
  new ConfigEntry {
    name = "design"
    basePath = "design-system"
  }
}
"#,
    )
    .unwrap();

    let loaded = load_config_entry_ignores_with_source(Some(&config_path));

    assert_eq!(loaded.source_path.as_deref(), Some(config_path.as_path()));
    assert_eq!(loaded.ignores.len(), 2);
    assert_eq!(loaded.ignores[0].base_path, None);
    assert_eq!(loaded.ignores[0].pattern.as_str(), "src/generated.ts");
    assert_eq!(loaded.ignores[1].base_path, None);
    assert_eq!(loaded.ignores[1].pattern.as_str(), "components/Legacy.vue");
}

#[test]
fn loads_json_type_checker_settings() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{
          "typeChecker": {
            "strict": true,
            "globalsFile": "./globals.d.ts"
          },
          "formatter": {
            "singleQuote": true,
            "printWidth": 88
          }
        }"#,
    )
    .unwrap();

    let config = load_config(Some(dir.path()));

    insta::assert_snapshot!(serde_json::to_string_pretty(&config).unwrap());
}

#[test]
fn loads_json_legacy_vue2_feature_flags() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{
          "typeChecker": {
            "legacyVue2": true
          },
          "languageServer": {
            "legacyVue2": true
          }
        }"#,
    )
    .unwrap();

    let loaded = load_config_with_features_and_source(Some(dir.path()));

    assert!(loaded.features.type_checker_legacy_vue2);
    assert_eq!(loaded.features.language_server_legacy_vue2, Some(true));
    assert!(loaded.config.type_checker.enabled);
}

#[test]
fn loads_json_vue_version_dialect() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{
          "vue": {
            "version": "0.10"
          }
        }"#,
    )
    .unwrap();

    let loaded = load_config_with_features_and_source(Some(dir.path()));

    assert_eq!(loaded.features.vue_version, Some(VueVersion::V0_10));
}

#[test]
fn vue_version_defaults_to_unset() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("vize.config.json"), r#"{}"#).unwrap();

    let loaded = load_config_with_features_and_source(Some(dir.path()));

    assert_eq!(loaded.features.vue_version, None);
}

#[test]
fn rejects_ambiguous_vue_version() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.json");
    std::fs::write(
        &config_path,
        r#"{
          "vue": {
            "version": "0"
          }
        }"#,
    )
    .unwrap();

    // Explicit --config must hard-error with the actionable message.
    let error = validate_explicit_config_path(&config_path).unwrap_err();
    assert!(error.contains("ambiguous"), "{error}");
    assert!(error.contains("0.10") && error.contains("0.11"), "{error}");

    // Auto-discovery warns and falls back to defaults: it must never resolve
    // an ambiguous selector to some 0.x line.
    let loaded = load_config_with_features_and_source(Some(dir.path()));
    assert_eq!(loaded.features.vue_version, None);
}

#[test]
fn rejects_unquoted_vue_version_numbers() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.json");
    std::fs::write(
        &config_path,
        r#"{
          "vue": {
            "version": 2.7
          }
        }"#,
    )
    .unwrap();

    let error = validate_explicit_config_path(&config_path).unwrap_err();
    assert!(error.contains("Vue version string"), "{error}");
}

#[test]
fn loads_legacy_json_lsp_alias() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{
          "lsp": {
            "completion": false,
            "tsgo": false
          }
        }"#,
    )
    .unwrap();

    let config = load_config(Some(dir.path()));

    insta::assert_snapshot!(serde_json::to_string_pretty(&config).unwrap());
}

fn install_pkl_modules(root: &std::path::Path) {
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../npm/cli/pkl");
    let target = root.join("node_modules/vize/pkl");
    copy_dir_recursive(&source, &target);
}

fn copy_dir_recursive(source: &std::path::Path, target: &std::path::Path) {
    std::fs::create_dir_all(target).unwrap();

    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let entry_path = entry.path();
        let target_path = target.join(entry.file_name());

        if entry.file_type().unwrap().is_dir() {
            copy_dir_recursive(&entry_path, &target_path);
        } else {
            std::fs::copy(&entry_path, &target_path).unwrap();
        }
    }
}

#[test]
fn loads_legacy_json_aliases() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{
          "fmt": {
            "singleQuote": true
          },
          "check": {
            "globals": "./globals.d.ts"
          }
        }"#,
    )
    .unwrap();

    let config = load_config(Some(dir.path()));

    insta::assert_snapshot!(serde_json::to_string_pretty(&config).unwrap());
}

#[test]
fn loads_legacy_pkl_lsp_alias() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.pkl");
    install_pkl_modules(dir.path());
    std::fs::write(
        &config_path,
        r#"
amends "node_modules/vize/pkl/VizeConfig.pkl"

lsp {
  completion = false
  tsgo = false
}
"#,
    )
    .unwrap();

    let config = load_config(Some(dir.path()));

    insta::assert_snapshot!(serde_json::to_string_pretty(&config).unwrap());
}

#[test]
fn invalid_pkl_config_does_not_fall_back_to_json() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.pkl");
    install_pkl_modules(dir.path());
    std::fs::write(
        &config_path,
        r#"
amends "node_modules/vize/pkl/VizeConfig.pkl"

formatter {
  singleQuote =
}
"#,
    )
    .unwrap();
    std::fs::write(
        dir.path().join("vize.config.json"),
        r#"{ "formatter": { "singleQuote": true } }"#,
    )
    .unwrap();

    let loaded = load_config_with_source(Some(dir.path()));

    assert_eq!(loaded.source_path.as_deref(), Some(config_path.as_path()));
    assert!(
        !loaded.config.formatter.single_quote,
        "invalid higher-priority pkl config must not silently fall back to json",
    );
}
