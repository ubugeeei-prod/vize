//! Batch checking and editor admission share compiler-option dependency errors.
use crate::VirtualProject;
use vize_carton::cstr;

#[test]
fn missing_relative_and_package_extends_reject_only_checked_editor_admission() {
    for extends in ["./missing.json", "@fixture/missing/tsconfig.json"] {
        let root = tempfile::tempdir().unwrap();
        let config = root.path().join("tsconfig.json");
        std::fs::write(
            &config,
            serde_json::to_vec(&serde_json::json!({"extends":extends})).unwrap(),
        )
        .unwrap();
        let mut project = VirtualProject::new(root.path()).unwrap();
        project.set_tsconfig_path(Some(config.clone()));
        assert_eq!(project.governing_config_paths(), vec![config.clone()]);
        let error = project.checked_config_chain_paths().unwrap_err();
        assert_eq!(
            cstr!("{error}"),
            cstr!(
                "Configuration error: cannot resolve tsconfig extends {extends:?} from {}",
                config.display()
            )
        );
    }
}

#[test]
fn checked_admission_uses_array_and_package_resolution_and_reports_the_whole_graph() {
    let root = tempfile::tempdir().unwrap();
    let package = root.path().join("node_modules/@fixture/base");
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"tsconfig":"policy.json"}"#,
    )
    .unwrap();
    std::fs::write(
        package.join("policy.json"),
        r#"{"compilerOptions":{"strict":true}}"#,
    )
    .unwrap();
    let relative = root.path().join("relative.json");
    std::fs::write(&relative, "{} ").unwrap();
    let config = root.path().join("tsconfig.json");
    std::fs::write(
        &config,
        r#"{"extends":["./relative.json","@fixture/base"]}"#,
    )
    .unwrap();
    let mut project = VirtualProject::new(root.path()).unwrap();
    project.set_tsconfig_path(Some(config.clone()));
    let mut expected = vec![
        config,
        relative,
        vize_carton::path::canonicalize_non_verbatim(&package.join("policy.json")),
    ];
    expected.sort();
    assert_eq!(project.checked_config_chain_paths().unwrap(), expected);
    std::fs::remove_file(package.join("policy.json")).unwrap();
    assert!(project.checked_config_chain_paths().is_err());
}

#[test]
fn malformed_inherited_json_rejects_checked_admission_and_recreation_repairs_it() {
    let root = tempfile::tempdir().unwrap();
    let base = root.path().join("base.json");
    let config = root.path().join("tsconfig.json");
    std::fs::write(&config, r#"{"extends":"./base.json"}"#).unwrap();
    std::fs::write(&base, "{").unwrap();
    let mut project = VirtualProject::new(root.path()).unwrap();
    project.set_tsconfig_path(Some(config.clone()));
    assert!(project.checked_config_chain_paths().is_err());
    std::fs::write(&base, "{}").unwrap();
    let mut expected = vec![config, base];
    expected.sort();
    assert_eq!(project.checked_config_chain_paths().unwrap(), expected);
}

#[test]
fn batch_materialization_and_snapshots_preserve_missing_extends_errors() {
    for extends in [
        "./tsconfig.base.json",
        "@fixture/missing",
        "@fixture/missing/tsconfig.json",
    ] {
        for entry in [serde_json::json!(extends), serde_json::json!([extends])] {
            let root = tempfile::tempdir().unwrap();
            let config = root.path().join("tsconfig.json");
            std::fs::write(
                &config,
                serde_json::to_vec(&serde_json::json!({
                    "extends":entry,"compilerOptions":{"strict":true}
                }))
                .unwrap(),
            )
            .unwrap();
            let mut project = VirtualProject::new(root.path()).unwrap();
            project.set_tsconfig_path(Some(config.clone()));
            let expected = cstr!(
                "Configuration error: cannot resolve tsconfig extends {extends:?} from {}",
                config.display()
            );
            for error in [
                project.materialize().unwrap_err(),
                project.checked_config_chain_paths().unwrap_err(),
                project
                    .load_compiler_options_verbatim(Some(&config))
                    .unwrap_err(),
                crate::snapshot_tsconfig_compiler_options(root.path(), &config).unwrap_err(),
            ] {
                assert_eq!(cstr!("{error}"), expected);
            }
        }
    }
}

#[test]
fn absent_implicit_config_keeps_defaults_but_an_explicit_missing_config_is_named() {
    let root = tempfile::tempdir().unwrap();
    let mut project = VirtualProject::new(root.path()).unwrap();
    assert!(project.checked_config_chain_paths().unwrap().is_empty());
    project.materialize().unwrap();

    let config = root.path().join("missing.json");
    project.set_tsconfig_path(Some(config.clone()));
    for error in [
        project.materialize().unwrap_err(),
        project.checked_config_chain_paths().unwrap_err(),
        crate::snapshot_tsconfig_compiler_options(root.path(), &config).unwrap_err(),
    ] {
        let message = cstr!("{error}");
        assert!(
            message.starts_with(
                cstr!(
                    "Configuration error: cannot read tsconfig {}:",
                    config.display()
                )
                .as_str()
            ),
            "{message}"
        );
    }
}

#[test]
fn deleted_relative_and_package_extends_fail_until_recreated_with_new_options() {
    for (extends, base_name) in [
        ("./tsconfig.base.json", "tsconfig.base.json"),
        ("@fixture/base", "node_modules/@fixture/base/policy.json"),
    ] {
        let root = tempfile::tempdir().unwrap();
        let base = root.path().join(base_name);
        std::fs::create_dir_all(base.parent().unwrap()).unwrap();
        if extends.starts_with('@') {
            std::fs::write(
                base.parent().unwrap().join("package.json"),
                r#"{"tsconfig":"policy.json"}"#,
            )
            .unwrap();
        }
        std::fs::write(&base, r#"{"compilerOptions":{"strict":true}}"#).unwrap();
        let config = root.path().join("tsconfig.json");
        std::fs::write(
            &config,
            serde_json::to_vec(&serde_json::json!({"extends":extends})).unwrap(),
        )
        .unwrap();
        let mut project = VirtualProject::new(root.path()).unwrap();
        project.set_tsconfig_path(Some(config.clone()));
        project.materialize().unwrap();
        assert_eq!(
            crate::snapshot_tsconfig_compiler_options(root.path(), &config).unwrap()["strict"],
            serde_json::json!(true)
        );
        std::fs::remove_file(&base).unwrap();
        assert_eq!(
            cstr!("{}", project.materialize().unwrap_err()),
            cstr!(
                "Configuration error: cannot resolve tsconfig extends {extends:?} from {}",
                config.display()
            )
        );
        std::fs::write(&base, r#"{"compilerOptions":{"strict":false}}"#).unwrap();
        project.materialize().unwrap();
        assert_eq!(
            crate::snapshot_tsconfig_compiler_options(root.path(), &config).unwrap()["strict"],
            serde_json::json!(false)
        );
    }
}

#[test]
fn selected_reference_owner_reports_its_extends_without_admitting_an_unrelated_owner() {
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("tsconfig.json");
    std::fs::write(
        &config,
        r#"{"files":[],"references":[{"path":"./tsconfig.app.json"},{"path":"./tsconfig.node.json"}]}"#,
    )
    .unwrap();
    let app_config = root.path().join("tsconfig.app.json");
    std::fs::write(
        &app_config,
        r#"{"extends":"./tsconfig.base.json","include":["App.vue"]}"#,
    )
    .unwrap();
    let app_config = vize_carton::path::canonicalize_non_verbatim(&app_config);
    std::fs::write(
        root.path().join("tsconfig.node.json"),
        r#"{"extends":"./unrelated-missing.json","include":["vite.config.ts"]}"#,
    )
    .unwrap();
    let source = root.path().join("App.vue");
    std::fs::write(&source, "<template />").unwrap();
    let mut project = VirtualProject::new(root.path()).unwrap();
    project.set_tsconfig_path(Some(config));
    project.use_effective_tsconfig_for_source(&source);
    assert_eq!(
        project.effective_tsconfig_path(),
        Some(vize_carton::path::canonicalize_non_verbatim(&app_config))
    );
    assert_eq!(
        cstr!("{}", project.materialize().unwrap_err()),
        cstr!(
            "Configuration error: cannot resolve tsconfig extends \"./tsconfig.base.json\" from {}",
            app_config.display()
        )
    );
    std::fs::write(
        root.path().join("tsconfig.base.json"),
        r#"{"compilerOptions":{"strict":true}}"#,
    )
    .unwrap();
    project.materialize().unwrap();
}
