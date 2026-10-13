//! `extends` composition across config files (#8894).

use std::path::Path;

use super::{
    load_config_and_linter_plan_with_lint_features_and_source, load_config_entry_files_with_source,
    load_linter_config, validate_explicit_config_path,
};
use crate::config::LintRuleSeverity;

fn write(path: &Path, contents: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, contents).unwrap();
}

fn rule(linter: &crate::config::LinterConfig, name: &str) -> Option<LintRuleSeverity> {
    linter.rules.get(name).copied()
}

#[test]
fn applies_a_relative_base_from_the_issue_report() {
    let root = tempfile::tempdir().unwrap();
    write(
        &root.path().join("base.json"),
        r#"{ "linter": { "enabled": true, "preset": "incremental", "rules": { "vue/no-v-html": "error" } } }"#,
    );
    let package = root.path().join("pkg");
    write(
        &package.join("vize.config.json"),
        r#"{ "extends": "../base.json" }"#,
    );

    let linter = load_linter_config(Some(&package));

    assert!(linter.enabled);
    assert_eq!(linter.preset.as_deref(), Some("incremental"));
    assert_eq!(
        rule(&linter, "vue/no-v-html"),
        Some(LintRuleSeverity::Error)
    );
}

#[test]
fn keeps_base_rules_when_the_extending_config_sets_its_own_preset() {
    let root = tempfile::tempdir().unwrap();
    write(
        &root.path().join("base.json"),
        r#"{ "linter": { "enabled": true, "preset": "incremental", "rules": { "vue/no-v-html": "error" } } }"#,
    );
    let package = root.path().join("pkg");
    write(
        &package.join("vize.config.json"),
        r#"{ "extends": ["../base.json"], "linter": { "enabled": true, "preset": "incremental" } }"#,
    );

    let linter = load_linter_config(Some(&package));

    assert_eq!(
        rule(&linter, "vue/no-v-html"),
        Some(LintRuleSeverity::Error)
    );
}

#[test]
fn applies_bases_in_order_and_lets_the_extending_config_win() {
    let root = tempfile::tempdir().unwrap();
    write(
        &root.path().join("first.json"),
        r#"{ "linter": { "preset": "nuxt", "rules": { "a": "error", "b": "error", "c": "error" } } }"#,
    );
    write(
        &root.path().join("second.json"),
        r#"{ "linter": { "preset": "incremental", "rules": { "b": "warn" } } }"#,
    );
    write(
        &root.path().join("vize.config.json"),
        r#"{ "extends": ["./first.json", "./second.json"], "linter": { "rules": { "c": "off" } } }"#,
    );

    let linter = load_linter_config(Some(root.path()));

    assert_eq!(linter.preset.as_deref(), Some("incremental"));
    assert_eq!(rule(&linter, "a"), Some(LintRuleSeverity::Error));
    assert_eq!(rule(&linter, "b"), Some(LintRuleSeverity::Warn));
    assert_eq!(rule(&linter, "c"), Some(LintRuleSeverity::Off));
}

#[test]
fn composes_nested_bases_relative_to_each_file() {
    let root = tempfile::tempdir().unwrap();
    write(
        &root.path().join("shared/core/core.json"),
        r#"{ "linter": { "rules": { "core": "error", "shared": "error" } } }"#,
    );
    write(
        &root.path().join("shared/base.json"),
        r#"{ "extends": "./core/core.json", "linter": { "rules": { "shared": "warn" } } }"#,
    );
    write(
        &root.path().join("app/vize.config.json"),
        r#"{ "extends": "../shared/base.json" }"#,
    );

    let linter = load_linter_config(Some(&root.path().join("app")));

    assert_eq!(rule(&linter, "core"), Some(LintRuleSeverity::Error));
    assert_eq!(rule(&linter, "shared"), Some(LintRuleSeverity::Warn));
}

#[test]
fn resolves_a_directory_base_to_its_config_file() {
    let root = tempfile::tempdir().unwrap();
    write(
        &root.path().join("shared/vize.config.json"),
        r#"{ "linter": { "rules": { "from-directory": "error" } } }"#,
    );
    write(
        &root.path().join("app/vize.config.json"),
        r#"{ "extends": "../shared" }"#,
    );

    let linter = load_linter_config(Some(&root.path().join("app")));

    assert_eq!(
        rule(&linter, "from-directory"),
        Some(LintRuleSeverity::Error)
    );
}

#[test]
fn appends_the_extending_entries_after_the_base_entries() {
    let root = tempfile::tempdir().unwrap();
    write(
        &root.path().join("base.json"),
        r#"{ "entries": [{ "files": ["base/**/*.vue"] }] }"#,
    );
    write(
        &root.path().join("vize.config.json"),
        r#"{ "extends": "./base.json", "entries": [{ "files": ["own/**/*.vue"] }] }"#,
    );

    let loaded = load_config_entry_files_with_source(Some(root.path()));
    let files: Vec<&str> = loaded
        .entries
        .iter()
        .flat_map(|entry| entry.files.iter().map(|file| file.as_str()))
        .collect();

    assert_eq!(files, ["base/**/*.vue", "own/**/*.vue"]);
}

#[test]
fn an_entry_extends_settings_but_keeps_its_own_scope() {
    let root = tempfile::tempdir().unwrap();
    write(
        &root.path().join("strict.json"),
        r#"{ "files": ["ignored/**"], "linter": { "rules": { "strict": "error", "shared": "error" } } }"#,
    );
    write(
        &root.path().join("vize.config.json"),
        r#"{
  "linter": { "rules": { "shared": "warn" } },
  "entries": [
    { "files": ["src/**/*.vue"], "extends": "./strict.json", "linter": { "rules": { "shared": "off" } } }
  ]
}"#,
    );

    let (_, plan, _) = load_config_and_linter_plan_with_lint_features_and_source(Some(root.path()));

    assert_eq!(plan.entries.len(), 1);
    assert_eq!(
        plan.entries[0].files.as_deref().unwrap()[0].as_str(),
        "src/**/*.vue"
    );
    let resolved = plan.resolve_matching_entries(&[0]);
    assert_eq!(
        resolved.rules.get("strict").copied(),
        Some(LintRuleSeverity::Error)
    );
    assert_eq!(
        resolved.rules.get("shared").copied(),
        Some(LintRuleSeverity::Off)
    );
}

#[test]
fn rejects_an_entry_that_extends_a_config_with_entries() {
    let root = tempfile::tempdir().unwrap();
    write(
        &root.path().join("multi.json"),
        r#"{ "entries": [{ "files": ["a/**"] }] }"#,
    );
    let config = root.path().join("vize.config.json");
    write(
        &config,
        r#"{ "entries": [{ "files": ["src/**"], "extends": "./multi.json" }] }"#,
    );

    let error = validate_explicit_config_path(&config).unwrap_err();

    assert!(
        error.contains("cannot extend ./multi.json"),
        "unexpected error: {error}"
    );
}

#[test]
fn reports_a_cycle_instead_of_recursing() {
    let root = tempfile::tempdir().unwrap();
    write(&root.path().join("a.json"), r#"{ "extends": "./b.json" }"#);
    write(
        &root.path().join("b.json"),
        r#"{ "extends": "./vize.config.json" }"#,
    );
    let config = root.path().join("vize.config.json");
    write(&config, r#"{ "extends": "./a.json" }"#);

    let error = validate_explicit_config_path(&config).unwrap_err();

    assert!(
        error.contains("Circular extends"),
        "unexpected error: {error}"
    );
}

#[test]
fn reports_a_missing_base() {
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("vize.config.json");
    write(&config, r#"{ "extends": "./missing.json" }"#);

    let error = validate_explicit_config_path(&config).unwrap_err();

    assert!(
        error.contains("cannot find the extended config ./missing.json"),
        "unexpected error: {error}"
    );
}

#[test]
fn rejects_an_extends_value_that_is_not_a_string() {
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("vize.config.json");
    write(&config, r#"{ "extends": [1] }"#);

    let error = validate_explicit_config_path(&config).unwrap_err();

    assert!(
        error.contains("extends must be a string or an array of strings"),
        "unexpected error: {error}"
    );
}

#[test]
fn does_not_change_configs_that_only_mention_extends_in_values() {
    let root = tempfile::tempdir().unwrap();
    write(
        &root.path().join("vize.config.json"),
        r#"{ "linter": { "rules": { "extends": "error" } } }"#,
    );

    let linter = load_linter_config(Some(root.path()));

    assert_eq!(rule(&linter, "extends"), Some(LintRuleSeverity::Error));
}

#[cfg(not(target_arch = "wasm32"))]
mod node {
    use super::*;

    fn node_available() -> bool {
        std::process::Command::new("node")
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success())
    }

    #[test]
    fn resolves_a_scoped_package_subpath() {
        if !node_available() {
            return;
        }
        let root = tempfile::tempdir().unwrap();
        let package = root.path().join("node_modules/@acme/lint-config");
        write(
            &package.join("package.json"),
            r#"{ "name": "@acme/lint-config", "exports": { "./vize": "./vize.json" } }"#,
        );
        write(
            &package.join("vize.json"),
            r#"{ "linter": { "rules": { "from-package": "error" } } }"#,
        );
        write(
            &root.path().join("vize.config.json"),
            r#"{ "extends": "@acme/lint-config/vize" }"#,
        );

        let linter = load_linter_config(Some(root.path()));

        assert_eq!(rule(&linter, "from-package"), Some(LintRuleSeverity::Error));
    }

    #[test]
    fn a_javascript_config_extends_a_json_base() {
        if !node_available() {
            return;
        }
        let root = tempfile::tempdir().unwrap();
        write(
            &root.path().join("base.json"),
            r#"{ "linter": { "rules": { "from-base": "error" } } }"#,
        );
        write(
            &root.path().join("vize.config.mjs"),
            r#"export default { extends: "./base.json", linter: { rules: { own: "warn" } } };"#,
        );

        let linter = load_linter_config(Some(root.path()));

        assert_eq!(rule(&linter, "from-base"), Some(LintRuleSeverity::Error));
        assert_eq!(rule(&linter, "own"), Some(LintRuleSeverity::Warn));
    }
}
