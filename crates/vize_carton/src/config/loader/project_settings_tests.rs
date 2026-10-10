use super::{
    load_config_with_features_and_source, load_lsp_config_snapshot, try_load_formatter_snapshot,
};

#[test]
fn native_vite_settings_use_the_same_projection_as_public_js() {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("vite.config.mjs"),
        r#"
        export default {
            vize: { languageServer: { formatting: false }, typeChecker: { jsxTypecheck: false } },
            compiler: { whitespace: "preserve", vueVersion: 2.7 },
            typecheck: { tsconfig: "tsconfig.app.json" },
            lint: { vize: { preset: "essential" } },
            fmt: { vize: { printWidth: 90, singleQuote: true } },
        };
    "#,
    )
    .unwrap();
    let loaded = load_lsp_config_snapshot(Some(project.path()));
    assert_eq!(
        loaded.source_path.unwrap(),
        project.path().join("vite.config.mjs")
    );
    assert_eq!(
        loaded.config.type_checker.tsconfig.as_deref(),
        Some("tsconfig.app.json")
    );
    assert!(!loaded.features.type_checker_jsx_typecheck);
    assert_eq!(loaded.linter.preset.as_deref(), Some("essential"));
    assert!(loaded.config.formatter.single_quote);
    assert_eq!(loaded.config.formatter.print_width, 90);
    assert_eq!(loaded.config.language_server.formatting, Some(false));
}

#[test]
fn dedicated_config_precedes_vite_and_preserves_existing_defaults() {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("vite.config.mjs"),
        "export default { fmt: { vize: { singleQuote: true } } };",
    )
    .unwrap();
    let fresh = load_config_with_features_and_source(Some(project.path()));
    assert!(fresh.config.formatter.single_quote);
    assert!(fresh.features.type_checker_jsx_typecheck);
    std::fs::write(project.path().join("vize.config.json"), "{}").unwrap();
    let compatible = load_config_with_features_and_source(Some(project.path()));
    assert!(!compatible.config.formatter.single_quote);
    assert!(!compatible.features.type_checker_jsx_typecheck);
    assert_eq!(
        compatible.source_path.unwrap(),
        project.path().join("vize.config.json")
    );
}

#[test]
fn malformed_vite_settings_fail_closed_before_formatter_writes() {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("vite.config.mjs"),
        "export default { fmt: { vize: { tabWidth: 'wrong' } } };",
    )
    .unwrap();
    let error = try_load_formatter_snapshot(Some(project.path())).unwrap_err();
    assert!(error.contains("vite.config.mjs"), "{error}");
}

#[test]
fn automatic_discovery_has_package_boundaries_but_explicit_roots_stay_shallow() {
    let project = tempfile::tempdir().unwrap();
    let package = project.path().join("packages/app");
    let source = package.join("src");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::write(package.join("tsconfig.json"), "{}").unwrap();
    let directories = super::discovery::search_directories(&source, true);
    assert_eq!(directories, vec![source.clone(), package]);
    assert_eq!(
        super::discovery::search_directories(&source, false),
        vec![source]
    );
}

#[test]
fn invalid_dedicated_editor_config_is_not_a_fresh_project_default() {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("vize.config.json"), "not valid json").unwrap();
    let loaded = load_lsp_config_snapshot(Some(project.path()));
    assert!(!loaded.valid);
    assert_eq!(loaded.source_path, None);
    assert!(!loaded.features.type_checker_jsx_typecheck);
    assert_eq!(loaded.config.language_server.formatting, None);
}
