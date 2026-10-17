//! Retired custom inheritance must not compete with TypeScript project inheritance.
use super::{try_load_project_config_with_source, validate_explicit_config_path};

#[test]
fn custom_native_extends_refuses_without_evaluating_its_base() {
    for document in [
        r#"{"extends":"missing.json"}"#,
        r#"{"entries":[{"files":["*.vue"],"extends":"missing.json"}]}"#,
        r#"[{"extends":"missing.json"}]"#,
    ] {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("serialized-config.json");
        std::fs::write(&path, document).unwrap();
        let error = validate_explicit_config_path(&path).unwrap_err();
        assert!(
            error.contains("Custom config extends is no longer supported"),
            "{error}"
        );
        assert!(
            !error.contains("No such file"),
            "must refuse before reading a base: {error}"
        );
    }
}

#[test]
fn retired_dedicated_config_requires_migration_instead_of_silently_overriding_defaults() {
    let root = tempfile::tempdir().unwrap();
    let retired = root.path().join("vize.config.json");
    std::fs::write(&retired, r#"{"formatter":{"singleQuote":true}}"#).unwrap();
    assert!(
        validate_explicit_config_path(&retired)
            .unwrap_err()
            .contains("no longer supported")
    );
    assert!(
        try_load_project_config_with_source(Some(root.path()))
            .unwrap_err()
            .contains("move Vue settings to vue:")
    );
    std::fs::write(
        root.path().join("vite.config.mjs"),
        "export default {vue:{formatter:{singleQuote:false}}};",
    )
    .unwrap();
    let config = try_load_project_config_with_source(Some(root.path())).unwrap();
    assert!(
        !config
            .document
            .into_config_and_features()
            .0
            .formatter
            .single_quote
    );
    assert_eq!(
        config.source_path.unwrap().file_name().unwrap(),
        "vite.config.mjs"
    );
}
