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
        assert_eq!(
            error,
            format!(
                "failed to parse {}: Custom config extends is no longer supported; use defineConfig(vpConfig).withVue(vueConfig); standard tsconfig extends remains supported",
                path.display()
            )
        );
    }
}

#[test]
fn retired_dedicated_config_requires_migration_instead_of_silently_overriding_defaults() {
    let root = tempfile::tempdir().unwrap();
    let retired = root.path().join("vize.config.json");
    std::fs::write(&retired, r#"{"formatter":{"singleQuote":true}}"#).unwrap();
    assert_eq!(
        validate_explicit_config_path(&retired).unwrap_err(),
        format!(
            "failed to parse {}: {}",
            retired.display(),
            super::discovery::CONFIG_MIGRATION
        )
    );
    assert_eq!(
        try_load_project_config_with_source(Some(root.path())).unwrap_err(),
        super::discovery::CONFIG_MIGRATION
    );
    std::fs::write(
        root.path().join("vite.config.mjs"),
        "export default {vue:{formatter:{singleQuote:false}}};",
    )
    .unwrap();
    assert_eq!(
        try_load_project_config_with_source(Some(root.path())).unwrap_err(),
        super::discovery::CONFIG_MIGRATION
    );
}
