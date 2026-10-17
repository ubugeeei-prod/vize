use super::validate_tsconfig_chain;
use std::{fs, path::Path};
use vize_l0::cstr;

fn missing_error(config: &Path, dependency: &str) -> vize_l0::String {
    cstr!(
        "Failed to resolve TypeScript configuration `{}`: Configuration error: cannot resolve tsconfig extends {dependency:?} from {}",
        config.display(),
        config.display()
    )
}

#[test]
fn missing_inherited_base_is_named_and_recreation_recovers_without_source_edit() {
    let case = tempfile::tempdir().unwrap();
    let config = case.path().join("tsconfig.json");
    let base = case.path().join("tsconfig.base.json");
    let source = case.path().join("TypedPanel.vue");
    let bytes = "<script setup lang=\"ts\">\nconst value: string = null;\n</script>\n<template><p>{{ value }}</p></template>\n";
    fs::write(&source, bytes).unwrap();
    fs::write(
        &config,
        r#"{"extends":"./tsconfig.base.json","include":["TypedPanel.vue"]}"#,
    )
    .unwrap();
    let config_before = fs::read(&config).unwrap();
    assert_eq!(
        validate_tsconfig_chain(&config).unwrap_err(),
        missing_error(&config, "./tsconfig.base.json")
    );
    fs::write(&base, r#"{"compilerOptions":{"strictNullChecks":false}}"#).unwrap();
    validate_tsconfig_chain(&config).unwrap();
    fs::remove_file(&base).unwrap();
    assert_eq!(
        validate_tsconfig_chain(&config).unwrap_err(),
        missing_error(&config, "./tsconfig.base.json")
    );
    fs::write(&base, r#"{"compilerOptions":{"strictNullChecks":true}}"#).unwrap();
    validate_tsconfig_chain(&config).unwrap();
    assert_eq!(fs::read(&source).unwrap(), bytes.as_bytes());
    assert_eq!(fs::read(&config).unwrap(), config_before);
}

#[test]
fn array_and_package_extends_use_the_same_strict_resolver() {
    let case = tempfile::tempdir().unwrap();
    let config = case.path().join("tsconfig.json");
    fs::write(case.path().join("present.json"), "{}").unwrap();
    for (content, dependency) in [
        (
            r#"{"extends":["./present.json","./missing.json"]}"#,
            "./missing.json",
        ),
        (
            r#"{"extends":"@missing/tsconfig/base.json"}"#,
            "@missing/tsconfig/base.json",
        ),
    ] {
        fs::write(&config, content).unwrap();
        assert_eq!(
            validate_tsconfig_chain(&config).unwrap_err(),
            missing_error(&config, dependency)
        );
    }
}

#[test]
fn nested_missing_dependency_names_the_selected_and_declaring_configs() {
    let case = tempfile::tempdir().unwrap();
    let config = case.path().join("tsconfig.json");
    let base = case.path().join("base.json");
    fs::write(&config, r#"{"extends":"./base.json"}"#).unwrap();
    fs::write(&base, r#"{"extends":"./missing.json"}"#).unwrap();
    assert_eq!(
        validate_tsconfig_chain(&config).unwrap_err(),
        cstr!(
            "Failed to resolve TypeScript configuration `{}`: Configuration error: cannot resolve tsconfig extends \"./missing.json\" from {}",
            config.display(),
            base.display()
        )
    );
}

#[test]
fn malformed_inherited_configuration_refuses_and_valid_repair_recovers() {
    let case = tempfile::tempdir().unwrap();
    let config = case.path().join("tsconfig.json");
    let base = case.path().join("base.json");
    fs::write(&config, r#"{"extends":"./base.json"}"#).unwrap();
    fs::write(&base, "{invalid").unwrap();
    let error = validate_tsconfig_chain(&config).unwrap_err();
    assert_eq!(
        error,
        cstr!(
            "Failed to resolve TypeScript configuration `{}`: JSON parse error: key must be a string at line 1 column 2",
            config.display()
        )
    );
    fs::write(&base, "{}").unwrap();
    validate_tsconfig_chain(&config).unwrap();
}

#[test]
fn explicit_missing_selected_config_cannot_fall_back_to_defaults() {
    let case = tempfile::tempdir().unwrap();
    let config = case.path().join("missing.json");
    assert_eq!(
        validate_tsconfig_chain(&config).unwrap_err(),
        cstr!(
            "TypeScript config file not found: `{}`; no files were checked.",
            config.display()
        )
    );
}

#[test]
fn canonical_diamond_and_cycle_policy_remains_accepted() {
    let case = tempfile::tempdir().unwrap();
    let config = case.path().join("tsconfig.json");
    fs::write(&config, r#"{"extends":["./left.json","./right.json"]}"#).unwrap();
    fs::write(
        case.path().join("left.json"),
        r#"{"extends":"./shared.json"}"#,
    )
    .unwrap();
    fs::write(
        case.path().join("right.json"),
        r#"{"extends":"./shared.json"}"#,
    )
    .unwrap();
    fs::write(case.path().join("shared.json"), "{}").unwrap();
    validate_tsconfig_chain(&config).unwrap();
    fs::write(
        case.path().join("shared.json"),
        r#"{"extends":"./tsconfig.json"}"#,
    )
    .unwrap();
    validate_tsconfig_chain(&config).unwrap();
}
