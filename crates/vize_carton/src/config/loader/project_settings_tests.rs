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
        project
            .path()
            .canonicalize()
            .unwrap()
            .join("tsconfig.app.json")
            .to_str()
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

#[test]
fn project_snapshot_resolves_vite_root_paths_from_one_evaluation() {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("vite.config.mjs"),
        r#"import { appendFileSync } from 'node:fs';
        export default () => {
            appendFileSync(new URL('.evaluations', import.meta.url), 'once');
            return {root:'app',vize:{
                typeChecker:{tsconfig:'tsconfig.json'}, ignores:['src/Ignored.vue'],
                entries:[{files:['src/*.vue'],linter:{rules:{'a11y/alt-text':'error'}}}]
            }};
        };"#,
    )
    .unwrap();
    let loaded = super::try_load_project_config_with_source(Some(project.path())).unwrap();
    let root = project.path().canonicalize().unwrap().join("app");
    assert_eq!(loaded.project_root.as_deref(), Some(root.as_path()));
    assert_eq!(
        std::fs::read_to_string(project.path().join(".evaluations")).unwrap(),
        "once"
    );
    let ignores = loaded.document.entry_ignores();
    assert_eq!(
        ignores
            .iter()
            .map(|ignore| (ignore.base_path.as_deref(), ignore.pattern.as_str()))
            .collect::<Vec<_>>(),
        vec![(root.to_str(), "src/Ignored.vue")]
    );
    let plan = loaded.document.linter_plan();
    assert_eq!(plan.entries[0].base_path.as_deref(), root.to_str());
    let (config, _) = loaded.document.into_config_and_features();
    assert_eq!(
        config.type_checker.tsconfig.as_deref(),
        root.join("tsconfig.json").to_str()
    );

    std::fs::write(
        project.path().join("vize.config.json"),
        "{\"projectRoot\":42,\"__vizeProjectRoot\":42}",
    )
    .unwrap();
    let dedicated = super::try_load_project_config_with_source(Some(project.path())).unwrap();
    assert_eq!(dedicated.project_root, None);
    assert_eq!(
        dedicated.source_path,
        Some(project.path().join("vize.config.json"))
    );

    std::fs::write(
        project.path().join("vize.config.json"),
        r#"{"__vizeProjectRoot":"untrusted","ignores":["src/Ignored.vue"]}"#,
    )
    .unwrap();
    let dedicated = super::try_load_project_config_with_source(Some(project.path())).unwrap();
    assert_eq!(dedicated.document.project_root(), None);
    assert_eq!(
        dedicated
            .document
            .entry_ignores()
            .iter()
            .map(|ignore| (ignore.base_path.as_deref(), ignore.pattern.as_str()))
            .collect::<Vec<_>>(),
        vec![(None, "src/Ignored.vue")]
    );
}

#[test]
fn native_array_projection_retains_global_settings_and_ordered_root_scopes() {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("vite.config.mjs"), "export default {root:'app',vize:[{__vizeProjectRoot:'decoy',formatter:{singleQuote:true},linter:{preset:'essential'}},{basePath:'ui',files:['*.vue'],linter:{rules:{'a11y/alt-text':'error'}}}]};").unwrap();
    let loaded = super::try_load_project_config_with_source(Some(project.path())).unwrap();
    let root = project.path().canonicalize().unwrap().join("app");
    assert_eq!(loaded.project_root.as_deref(), Some(root.as_path()));
    let plan = loaded.document.linter_plan();
    assert_eq!(plan.base.preset.as_deref(), Some("essential"));
    assert_eq!(
        plan.entries[0].base_path.as_deref(),
        root.join("ui").to_str()
    );
    let (config, _) = loaded.document.into_config_and_features();
    assert!(config.formatter.single_quote);
}

#[test]
fn editor_snapshot_keeps_lint_scopes_and_paths_from_one_host_evaluation() {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("vite.config.mjs"),
        r#"import { appendFileSync } from 'node:fs';
        export default () => {
            appendFileSync(new URL('.evaluations', import.meta.url), 'once');
            return {root:'app',vize:{
                typeChecker:{tsconfig:'tsconfig.json'}, ignores:['src/Ignored.vue'],
                entries:[{files:['src/*.vue'],linter:{rules:{'a11y/alt-text':'error'}}}]
            }};
        };"#,
    )
    .unwrap();
    let loaded = load_lsp_config_snapshot(Some(project.path()));
    let root = project.path().canonicalize().unwrap().join("app");
    let plan = loaded.project.document.linter_plan();
    assert_eq!(
        (
            loaded.valid,
            loaded.source_path.as_deref(),
            loaded.project.source_path.as_deref(),
            loaded.project.project_root.as_deref(),
            loaded.config.type_checker.tsconfig.as_deref(),
            plan.entries[0].base_path.as_deref(),
            loaded.project.document.entry_ignores(),
            std::fs::read_to_string(project.path().join(".evaluations")).unwrap(),
        ),
        (
            true,
            Some(project.path().join("vite.config.mjs").as_path()),
            Some(project.path().join("vite.config.mjs").as_path()),
            Some(root.as_path()),
            root.join("tsconfig.json").to_str(),
            root.to_str(),
            vec![super::ConfigEntryIgnore {
                base_path: root.to_str().map(Into::into),
                pattern: "src/Ignored.vue".into(),
            }],
            "once".into(),
        )
    );
}

#[test]
fn editor_snapshot_owns_dedicated_relative_tsconfig_projection() {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("vize.config.json"),
        r#"{"typeChecker":{"tsconfig":"tsconfig.app.json","jsxTypecheck":false}}"#,
    )
    .unwrap();
    let loaded = load_lsp_config_snapshot(Some(project.path()));
    assert_eq!(
        (
            loaded.valid,
            loaded.project.project_root,
            loaded.config.type_checker.tsconfig.as_deref(),
            loaded.features.type_checker_jsx_typecheck,
            loaded.config.language_server.formatting,
        ),
        (
            true,
            None,
            project.path().join("tsconfig.app.json").to_str(),
            false,
            None,
        )
    );
}

#[test]
fn empty_editor_workspace_is_valid_without_weakening_explicit_cli_selection() {
    let project = tempfile::tempdir().unwrap();
    let loaded = load_lsp_config_snapshot(Some(project.path()));
    assert_eq!(
        (
            loaded.valid,
            loaded.source_path,
            loaded.project.source_path,
            loaded.project.project_root,
            loaded.features.type_checker_jsx_typecheck,
        ),
        (true, None, None, None, true)
    );
    assert_eq!(
        super::try_load_project_config_with_source(Some(project.path())).unwrap_err(),
        format!(
            "no vize config file found under {}",
            project.path().display()
        )
    );
}

#[test]
fn empty_editor_directory_retains_fresh_defaults_and_strict_cli_checks() {
    let project = tempfile::tempdir().unwrap();
    let loaded = load_lsp_config_snapshot(Some(project.path()));
    assert_eq!(
        (
            loaded.valid,
            loaded.source_path,
            loaded.features.type_checker_jsx_typecheck,
            loaded.config.type_checker,
            loaded.config.language_server.formatting,
            loaded.request_timeout_ms,
        ),
        (
            true,
            None,
            true,
            crate::config::TypeCheckerConfig::default(),
            None,
            60_000
        )
    );
    assert_eq!(
        super::load_raw_config_checked(Some(project.path())).err(),
        Some(format!(
            "no vize config file found under {}",
            project.path().display()
        ))
    );
}
