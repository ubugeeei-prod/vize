//! Retained compiler authority remains executable without its live config chain.
#[expect(
    clippy::disallowed_types,
    reason = "these fixtures verify shared immutable compiler snapshot ownership"
)]
use super::{Arc, CompilerConfigurationSnapshot, VirtualProject};
use serde_json::json;

#[test]
#[expect(
    clippy::disallowed_types,
    reason = "the fixture shares admitted immutable snapshots between native project owners"
)]
fn admitted_options_keep_package_path_anchors_vue_options_and_live_source_rebuilds() {
    let root = tempfile::tempdir().unwrap();
    let package = root.path().join("node_modules/@fixture/policy");
    std::fs::create_dir_all(package.join("types")).unwrap();
    let package = package.canonicalize().unwrap();
    std::fs::write(package.join("package.json"), r#"{"tsconfig":"base.json"}"#).unwrap();
    let base = package.join("base.json");
    std::fs::write(&base, r#"{
        "compilerOptions":{"strict":true,"baseUrl":".","paths":{"@policy/*":["types/*"]},"typeRoots":["types"]},
        "vueCompilerOptions":{"strictTemplates":true,"jsxSlots":true}
    }"#).unwrap();
    let config = root.path().join("tsconfig.json");
    std::fs::write(
        &config,
        r#"{"extends":"@fixture/policy","compilerOptions":{"strictNullChecks":true}}"#,
    )
    .unwrap();
    let snapshot =
        Arc::new(CompilerConfigurationSnapshot::capture(root.path(), Some(&config)).unwrap());
    let expected = snapshot.flattened.options.clone();
    let dependencies = snapshot.dependencies().to_vec();
    assert_eq!(expected["strict"], json!(true));
    assert_eq!(expected["strictNullChecks"], json!(true));
    let target = package.join("types/*").to_string_lossy().replace('\\', "/");
    let roots = package.join("types").to_string_lossy().replace('\\', "/");
    assert_eq!(
        crate::snapshot_tsconfig_compiler_options(root.path(), &config).unwrap()["paths"]["@policy/*"],
        json!([target])
    );
    assert_eq!(
        crate::snapshot_tsconfig_compiler_options(root.path(), &config).unwrap()["typeRoots"],
        json!([roots])
    );
    std::fs::remove_file(&base).unwrap();
    let mut project = VirtualProject::new(root.path()).unwrap();
    project
        .set_compiler_configuration_snapshot(Arc::clone(&snapshot))
        .unwrap();
    assert_eq!(
        project.load_compiler_options(Some(&config)).unwrap(),
        expected
    );
    assert_eq!(project.governing_config_paths(), dependencies);
    assert!(project.checks_unknown_props());
    assert!(project.virtual_ts_check_options.jsx_slots);
    let source = root.path().join("App.vue");
    let original = "<script setup lang='ts'>const value: string = 'disk';</script>";
    std::fs::write(&source, original).unwrap();
    let unsaved = "<script setup lang='ts'>const value: number = 42;</script>";
    project
        .register_path_with_content(&source, unsaved)
        .unwrap();
    project.materialize().unwrap();
    assert!(
        project
            .find_by_original(&source)
            .unwrap()
            .content
            .contains("const value: number = 42")
    );
    assert_eq!(std::fs::read_to_string(&source).unwrap(), original);
    let generated: serde_json::Value =
        serde_json::from_slice(&std::fs::read(project.generated_tsconfig_path()).unwrap()).unwrap();
    assert_eq!(
        generated["compilerOptions"]["strictNullChecks"],
        json!(true)
    );
    // Direct loading skips the deleted base like `tsc`: only own options remain.
    let direct = crate::snapshot_tsconfig_compiler_options(root.path(), &config).unwrap();
    assert_eq!(direct["strictNullChecks"], json!(true));
    assert!(direct.get("strict").is_none());
}

#[test]
#[expect(
    clippy::disallowed_types,
    reason = "the fixture shares admitted immutable snapshots between native project owners"
)]
fn admitted_absence_does_not_adopt_a_new_invalid_live_config() {
    let root = tempfile::tempdir().unwrap();
    let snapshot = Arc::new(CompilerConfigurationSnapshot::capture(root.path(), None).unwrap());
    let config = root.path().join("tsconfig.json");
    std::fs::write(&config, r#"{"extends":"./missing.json"}"#).unwrap();
    let mut project = VirtualProject::new(root.path()).unwrap();
    project
        .set_compiler_configuration_snapshot(snapshot)
        .unwrap();
    assert!(project.effective_tsconfig_path().is_none());
    assert!(project.load_compiler_options(None).unwrap().is_empty());
    project.materialize().unwrap();
    // A live project reads the new config, reporting and skipping its base.
    let live = VirtualProject::new(root.path()).unwrap();
    live.materialize().unwrap();
    assert_eq!(live.unresolved_tsconfig_extends().len(), 1);
}

#[test]
fn inferred_capture_observes_but_does_not_adopt_an_existing_config() {
    let root = tempfile::tempdir().unwrap();
    let root = root.path().canonicalize().unwrap();
    let config = root.join("tsconfig.json");
    std::fs::write(&config, r#"{"compilerOptions":{"strict":true}}"#).unwrap();

    assert!(CompilerConfigurationSnapshot::capture(&root, None).is_err());
    let inferred = CompilerConfigurationSnapshot::capture_inferred_attempt(&root);
    assert!(inferred.dependencies.contains(&config));
    let inferred = inferred.outcome.unwrap();
    let configured = CompilerConfigurationSnapshot::capture(&root, Some(&config)).unwrap();

    assert!(inferred.tsconfig_path.is_none());
    assert!(inferred.flattened.options.is_empty());
    assert!(!inferred.same_authority(&configured));
}

#[test]
#[expect(
    clippy::disallowed_types,
    reason = "the fixture shares admitted immutable snapshots between native project owners"
)]
fn same_directory_selected_reference_snapshots_stay_independent_and_recover() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("tsconfig.json"),
        r#"{"files":[],"references":[{"path":"./legacy.json"},{"path":"./modern.json"}]}"#,
    )
    .unwrap();
    let base = root.path().join("base.json");
    std::fs::write(&base, r#"{"compilerOptions":{"strictNullChecks":true},"vueCompilerOptions":{"strictTemplates":true}}"#).unwrap();
    let legacy = root.path().join("legacy.json");
    let modern = root.path().join("modern.json");
    std::fs::write(
        &legacy,
        r#"{"extends":"./base.json","compilerOptions":{"strict":false},"include":["Legacy.vue"]}"#,
    )
    .unwrap();
    std::fs::write(&modern, r#"{"compilerOptions":{"strict":true,"strictNullChecks":false},"vueCompilerOptions":{"strictTemplates":false},"include":["Modern.vue"]}"#).unwrap();
    let before =
        Arc::new(CompilerConfigurationSnapshot::capture(root.path(), Some(&legacy)).unwrap());
    let sibling =
        Arc::new(CompilerConfigurationSnapshot::capture(root.path(), Some(&modern)).unwrap());
    std::fs::write(&base, "{").unwrap();
    assert!(CompilerConfigurationSnapshot::capture(root.path(), Some(&legacy)).is_err());
    for (snapshot, config, strict, nulls, unknown) in [
        (before, &legacy, false, true, true),
        (sibling, &modern, true, false, false),
    ] {
        let mut project = VirtualProject::new(root.path()).unwrap();
        project
            .set_compiler_configuration_snapshot(snapshot)
            .unwrap();
        let options = project.load_compiler_options(Some(config)).unwrap();
        assert_eq!(options["strict"], json!(strict));
        assert_eq!(options["strictNullChecks"], json!(nulls));
        assert_eq!(project.checks_unknown_props(), unknown);
        assert_eq!(
            project.effective_tsconfig_path().as_deref(),
            Some(config.as_path())
        );
        project.materialize().unwrap();
    }
    std::fs::write(&base, r#"{"compilerOptions":{"strictNullChecks":false},"vueCompilerOptions":{"strictTemplates":false}}"#).unwrap();
    let repaired =
        Arc::new(CompilerConfigurationSnapshot::capture(root.path(), Some(&legacy)).unwrap());
    let mut project = VirtualProject::new(root.path()).unwrap();
    project
        .set_compiler_configuration_snapshot(repaired)
        .unwrap();
    assert_eq!(
        project.load_compiler_options(Some(&legacy)).unwrap()["strictNullChecks"],
        json!(false)
    );
    assert!(!project.checks_unknown_props());
}

#[test]
fn concurrent_base_change_rejects_mixed_options_paths_and_vue_target() {
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("tsconfig.json");
    let base = root.path().join("base.json");
    std::fs::write(&config, r#"{"extends":"./base.json"}"#).unwrap();
    std::fs::write(&base, r#"{"compilerOptions":{"strict":true,"baseUrl":"old","paths":{"@policy/*":["types/*"]}},"vueCompilerOptions":{"target":2.7,"strictTemplates":true}}"#).unwrap();
    let changed = r#"{"compilerOptions":{"strict":false,"baseUrl":"new","paths":{"@policy/*":["types/*"]}},"vueCompilerOptions":{"target":3,"strictTemplates":false}}"#;
    let barrier = std::sync::Barrier::new(2);
    std::thread::scope(|scope| {
        let writer = scope.spawn(|| {
            barrier.wait();
            std::fs::write(&base, changed).unwrap();
            barrier.wait();
        });
        let result = CompilerConfigurationSnapshot::capture_with_observer(
            root.path(),
            Some(&config),
            false,
            || {
                barrier.wait();
                barrier.wait();
            },
        );
        assert!(
            vize_carton::cstr!("{}", result.err().unwrap())
                .contains("configuration changed while it was being read")
        );
        writer.join().unwrap();
    });
    let fresh = CompilerConfigurationSnapshot::capture(root.path(), Some(&config)).unwrap();
    assert_eq!(fresh.flattened.options["strict"], json!(false));
    assert_eq!(fresh.verbatim["strict"], json!(false));
    assert_eq!(
        fresh.flattened.options["paths"]["@policy/*"],
        json!(["new/types/*"])
    );
    assert_eq!(fresh.vue_options["strictTemplates"], json!(false));
    assert_eq!(
        fresh.vue_target().unwrap(),
        Some(vize_carton::config::VueVersion::V3)
    );
}

#[test]
fn package_manifest_retarget_rejects_byte_identical_configs_with_different_path_anchors() {
    let root = tempfile::tempdir().unwrap();
    let package = root.path().join("node_modules/policy");
    std::fs::create_dir_all(package.join("first")).unwrap();
    std::fs::create_dir_all(package.join("second")).unwrap();
    let manifest = package.join("package.json");
    std::fs::write(&manifest, r#"{"tsconfig":"first/base.json"}"#).unwrap();
    let policy = r#"{"compilerOptions":{"baseUrl":".","paths":{"@policy/*":["types/*"]}}}"#;
    for directory in ["first", "second"] {
        std::fs::write(package.join(directory).join("base.json"), policy).unwrap();
    }
    let config = root.path().join("tsconfig.json");
    std::fs::write(&config, r#"{"extends":"policy"}"#).unwrap();
    let result = CompilerConfigurationSnapshot::capture_with_observer(
        root.path(),
        Some(&config),
        false,
        || {
            std::fs::write(&manifest, r#"{"tsconfig":"second/base.json"}"#).unwrap();
        },
    );
    assert!(
        vize_carton::cstr!("{}", result.err().unwrap())
            .contains("configuration changed while it was being read")
    );
    let fresh = CompilerConfigurationSnapshot::capture(root.path(), Some(&config)).unwrap();
    assert_eq!(
        fresh.flattened.options["paths"]["@policy/*"],
        json!(["node_modules/policy/second/types/*"])
    );
    assert!(fresh.dependencies().contains(&manifest));
}

#[test]
#[cfg(unix)]
fn package_link_retarget_rejects_identical_config_bytes_at_a_new_real_anchor() {
    let root = tempfile::tempdir().unwrap();
    let modules = root.path().join("node_modules");
    std::fs::create_dir(&modules).unwrap();
    let policy = r#"{"compilerOptions":{"baseUrl":".","paths":{"@policy/*":["types/*"]}}}"#;
    for directory in ["first", "second"] {
        let package = root.path().join(directory);
        std::fs::create_dir(&package).unwrap();
        std::fs::write(package.join("tsconfig.json"), policy).unwrap();
    }
    let link = modules.join("policy");
    std::os::unix::fs::symlink(root.path().join("first"), &link).unwrap();
    let config = root.path().join("tsconfig.json");
    std::fs::write(&config, r#"{"extends":"policy"}"#).unwrap();
    let result = CompilerConfigurationSnapshot::capture_with_observer(
        root.path(),
        Some(&config),
        false,
        || {
            std::fs::remove_file(&link).unwrap();
            std::os::unix::fs::symlink(root.path().join("second"), &link).unwrap();
        },
    );
    assert!(
        vize_carton::cstr!("{}", result.err().unwrap())
            .contains("configuration changed while it was being read")
    );
    let fresh = CompilerConfigurationSnapshot::capture(root.path(), Some(&config)).unwrap();
    assert_eq!(
        fresh.flattened.options["paths"]["@policy/*"],
        json!(["second/types/*"])
    );
    assert!(fresh.dependencies().contains(&link));
}

#[test]
fn higher_priority_extends_candidate_creation_rejects_the_previous_resolution() {
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("tsconfig.json");
    std::fs::write(&config, r#"{"extends":"./policy"}"#).unwrap();
    std::fs::write(
        root.path().join("policy.json"),
        r#"{"compilerOptions":{"strict":true}}"#,
    )
    .unwrap();
    let preferred = root.path().join("policy");
    let result = CompilerConfigurationSnapshot::capture_with_observer(
        root.path(),
        Some(&config),
        false,
        || {
            std::fs::write(&preferred, r#"{"compilerOptions":{"strict":false}}"#).unwrap();
        },
    );
    assert!(
        vize_carton::cstr!("{}", result.err().unwrap())
            .contains("configuration changed while it was being read")
    );
    let fresh = CompilerConfigurationSnapshot::capture(root.path(), Some(&config)).unwrap();
    assert_eq!(fresh.flattened.options["strict"], json!(false));
    assert_eq!(fresh.verbatim["strict"], json!(false));
}

#[test]
fn captured_auto_vue_target_and_solution_shell_do_not_reread_live_configuration() {
    let root = tempfile::tempdir().unwrap();
    let vue = root.path().join("node_modules/vue");
    std::fs::create_dir_all(&vue).unwrap();
    let manifest = vue.join("package.json");
    std::fs::write(&manifest, r#"{"name":"vue","version":"2.7.16"}"#).unwrap();
    let config = root.path().join("tsconfig.json");
    std::fs::write(
        &config,
        r#"{"vueCompilerOptions":{"target":"auto"},"references":[{"path":"./app.json"}]}"#,
    )
    .unwrap();
    let referenced = root.path().join("app.json");
    std::fs::write(&referenced, "{}").unwrap();
    let captured = CompilerConfigurationSnapshot::capture(root.path(), Some(&config)).unwrap();
    assert_eq!(
        captured.vue_target().unwrap(),
        Some(vize_carton::config::VueVersion::V2_7)
    );
    assert!(captured.is_solution_shell());
    assert!(captured.dependencies().contains(&manifest));
    assert!(captured.dependencies().contains(&referenced));
    std::fs::write(&manifest, r#"{"name":"vue","version":"3.5.0"}"#).unwrap();
    std::fs::write(&config, r#"{"vueCompilerOptions":{"target":"auto"}}"#).unwrap();
    std::fs::remove_file(&referenced).unwrap();
    assert_eq!(
        captured.vue_target().unwrap(),
        Some(vize_carton::config::VueVersion::V2_7)
    );
    assert!(captured.is_solution_shell());
    let repaired = CompilerConfigurationSnapshot::capture(root.path(), Some(&config)).unwrap();
    assert_eq!(
        repaired.vue_target().unwrap(),
        Some(vize_carton::config::VueVersion::V3)
    );
    assert!(!repaired.is_solution_shell());
}

#[test]
fn admitted_absence_rejects_primary_or_fallback_configuration_creation_during_capture() {
    for name in ["tsconfig.json", "jsconfig.json"] {
        let root = tempfile::tempdir().unwrap();
        let candidate = root.path().join(name);
        let result =
            CompilerConfigurationSnapshot::capture_with_observer(root.path(), None, false, || {
                std::fs::write(&candidate, "{}").unwrap();
            });
        assert!(
            vize_carton::cstr!("{}", result.err().unwrap())
                .contains("configuration changed while it was being read")
        );
        let result = CompilerConfigurationSnapshot::capture(root.path(), None);
        assert!(
            vize_carton::cstr!("{}", result.err().unwrap())
                .contains("appeared while admitting configuration absence")
        );
        std::fs::remove_file(candidate).unwrap();
        assert!(CompilerConfigurationSnapshot::capture(root.path(), None).is_ok());
    }
}

#[test]
fn failed_capture_retains_new_external_extends_candidates_until_repaired() {
    let workspace = tempfile::tempdir().unwrap();
    let workspace = vize_carton::path::canonicalize_non_verbatim(workspace.path());
    let root = workspace.join("app");
    let outside = workspace.join("shared");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::create_dir_all(&outside).unwrap();
    let config = root.join("tsconfig.json");
    std::fs::write(&config, r#"{"extends":"../shared/new-base"}"#).unwrap();
    let crate::CompilerConfigurationAttempt {
        outcome,
        dependencies,
        ..
    } = CompilerConfigurationSnapshot::capture_attempt(&root, Some(&config));
    // The missing base is reported and skipped; its candidates stay watched.
    let admitted = outcome.expect("a missing base is skipped like tsc");
    assert_eq!(admitted.unresolved_extends().len(), 1);
    assert_eq!(
        admitted.unresolved_extends()[0].specifier,
        "../shared/new-base"
    );
    for candidate in [
        config.clone(),
        outside.join("new-base"),
        outside.join("new-base.json"),
        outside.join("new-base/tsconfig.json"),
    ] {
        assert!(dependencies.contains(&candidate), "{}", candidate.display());
    }
    assert!(dependencies.windows(2).all(|pair| pair[0] < pair[1]));
    let base = outside.join("new-base.json");
    let ancestor = outside.join("ancestor.json");
    std::fs::write(
        &base,
        r#"{"extends":"./ancestor.json","compilerOptions":{"strict":true}}"#,
    )
    .unwrap();
    std::fs::write(&ancestor, r#"{"vueCompilerOptions":{"jsxSlots":true}}"#).unwrap();
    let repaired = CompilerConfigurationSnapshot::capture_attempt(&root, Some(&config));
    let snapshot = repaired.outcome.unwrap();
    assert_eq!(repaired.dependencies, snapshot.dependencies());
    assert!(repaired.dependencies.contains(&base));
    assert!(repaired.dependencies.contains(&ancestor));
    assert_eq!(snapshot.flattened.options["strict"], json!(true));
    assert_eq!(snapshot.vue_options["jsxSlots"], json!(true));
    std::fs::remove_file(&ancestor).unwrap();
    let removed = CompilerConfigurationSnapshot::capture_attempt(&root, Some(&config));
    assert!(removed.dependencies.contains(&ancestor));
    let removed = removed.outcome.unwrap();
    assert_eq!(removed.unresolved_extends()[0].specifier, "./ancestor.json");
    assert_eq!(removed.flattened.options["strict"], json!(true));
    std::fs::write(&ancestor, "{}").unwrap();
    let restored = CompilerConfigurationSnapshot::capture(&root, Some(&config)).unwrap();
    assert!(restored.unresolved_extends().is_empty());
}

#[cfg(unix)]
#[test]
fn failed_package_capture_retains_missing_manifest_and_symlink_ancestor() {
    let workspace = tempfile::tempdir().unwrap();
    let workspace = vize_carton::path::canonicalize_non_verbatim(workspace.path());
    let root = workspace.join("app");
    let outside = workspace.join("policy");
    let scope = root.join("node_modules/@fixture");
    std::fs::create_dir_all(&scope).unwrap();
    std::fs::create_dir_all(&outside).unwrap();
    let linked = scope.join("policy");
    std::os::unix::fs::symlink(&outside, &linked).unwrap();
    let config = root.join("tsconfig.json");
    std::fs::write(&config, r#"{"extends":"@fixture/policy"}"#).unwrap();
    let attempted = CompilerConfigurationSnapshot::capture_attempt(&root, Some(&config));
    assert_eq!(
        attempted.outcome.as_ref().unwrap().unresolved_extends()[0].specifier,
        "@fixture/policy"
    );
    assert!(attempted.dependencies.contains(&linked));
    assert!(
        attempted
            .dependencies
            .contains(&linked.join("package.json"))
    );
    assert!(
        attempted
            .dependencies
            .contains(&linked.join("tsconfig.json"))
    );
    std::fs::write(outside.join("package.json"), r#"{"tsconfig":"base.json"}"#).unwrap();
    std::fs::write(
        outside.join("base.json"),
        r#"{"compilerOptions":{"strict":true}}"#,
    )
    .unwrap();
    let repaired = CompilerConfigurationSnapshot::capture_attempt(&root, Some(&config));
    let snapshot = repaired.outcome.unwrap();
    assert_eq!(repaired.dependencies, snapshot.dependencies());
    assert!(repaired.dependencies.contains(&linked));
    assert!(repaired.dependencies.contains(&linked.join("package.json")));
    assert!(repaired.dependencies.contains(&outside.join("base.json")));
    assert_eq!(snapshot.flattened.options["strict"], json!(true));
}

#[test]
fn rejected_absence_attempt_retains_both_default_config_candidates() {
    let root = tempfile::tempdir().unwrap();
    let root = vize_carton::path::canonicalize_non_verbatim(root.path());
    let config = root.join("tsconfig.json");
    let expected = vec![root.join("jsconfig.json"), config.clone()];
    let absent = CompilerConfigurationSnapshot::capture_attempt(&root, None);
    assert!(absent.outcome.is_ok());
    assert_eq!(absent.dependencies, expected);
    let raced =
        CompilerConfigurationSnapshot::capture_attempt_with_observer(&root, None, false, || {
            std::fs::write(&config, "{}").unwrap();
        });
    assert!(raced.outcome.is_err());
    assert!(raced.observations.needs_confirmation());
    assert_eq!(raced.dependencies, expected);
    let existing = CompilerConfigurationSnapshot::capture_attempt(&root, None);
    assert!(existing.outcome.is_err());
    assert!(existing.observations.needs_confirmation());
    assert_eq!(existing.dependencies, expected);
    assert!(CompilerConfigurationSnapshot::capture(&root, None).is_err());
}
