use vize_carton::config::{VueVersion, load_config_and_linter_with_lint_features_and_source};

#[test]
fn linter_features_read_compiler_compatibility_vue_version() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.json");
    std::fs::write(
        &config_path,
        r#"{ "compiler": { "compatibility": { "vueVersion": "2" } } }"#,
    )
    .unwrap();

    let (loaded, _, linter_features) =
        load_config_and_linter_with_lint_features_and_source(Some(&config_path));

    assert_eq!(loaded.features.vue_version, Some(VueVersion::V2));
    assert_eq!(linter_features.vue_version, Some(VueVersion::V2));
}

#[test]
fn linter_features_read_explicit_compiler_vapor_false() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.json");
    std::fs::write(&config_path, r#"{ "compiler": { "vapor": false } }"#).unwrap();

    let (_, _, linter_features) =
        load_config_and_linter_with_lint_features_and_source(Some(&config_path));

    assert_eq!(linter_features.vapor, Some(false));
}

#[test]
fn linter_features_read_legacy_vue2_flags() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("vize.config.json");
    std::fs::write(&config_path, r#"{ "typeChecker": { "legacyVue2": true } }"#).unwrap();

    let (loaded, _, linter_features) =
        load_config_and_linter_with_lint_features_and_source(Some(&config_path));

    assert!(loaded.features.type_checker_legacy_vue2);
    assert_eq!(linter_features.vue_version, Some(VueVersion::V2_7));
}
