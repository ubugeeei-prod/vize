use std::path::{Path, PathBuf};

use super::{TsconfigOwnershipOptions, TsconfigOwnershipSnapshot, TsconfigSourceKind};

fn write(root: &Path, file: &str, source: &str) -> PathBuf {
    let path = root.join(file);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, source).unwrap();
    path
}

#[test]
fn ownership_snapshot_options_and_membership_use_the_same_captured_revision() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().canonicalize().unwrap();
    let config = write(
        &root,
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"app.json"},{"path":"other.json"}]}"#,
    );
    let app = write(
        &root,
        "app.json",
        r#"{"include":["src/**/*"],"compilerOptions":{"paths":{"@/*":["old/*"]}}}"#,
    );
    write(&root, "other.json", r#"{"include":[]}"#);
    let source = root.join("src/App.vue");
    let snapshot = TsconfigOwnershipSnapshot::capture(&config).unwrap();
    assert_eq!(snapshot.select(&source, TsconfigSourceKind::Typed), app);
    write(
        &root,
        "app.json",
        r#"{"include":["src/**/*"],"compilerOptions":{"paths":{"@/*":["new/*"]}}}"#,
    );
    write(&root, "other.json", r#"{"include":["src/**/*"]}"#);
    assert_eq!(snapshot.select(&source, TsconfigSourceKind::Typed), app);
    let options = snapshot.compiler_options(&app).unwrap();
    assert_eq!(options.options["paths"]["@/*"][0], "old/*");
    assert!(snapshot.observations().needs_confirmation());
    let refreshed = TsconfigOwnershipSnapshot::capture(&config).unwrap();
    assert_eq!(refreshed.select(&source, TsconfigSourceKind::Typed), config);
    assert_eq!(
        refreshed.compiler_options(&app).unwrap().options["paths"]["@/*"][0],
        "new/*"
    );
    std::fs::remove_file(&app).unwrap();
    assert_eq!(
        snapshot.compiler_options(&app).unwrap().options["paths"]["@/*"][0],
        "old/*"
    );
}

#[test]
fn ownership_snapshot_options_keep_inherited_bytes_and_known_unresolved_extends() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().canonicalize().unwrap();
    let config = write(
        &root,
        "tsconfig.json",
        r#"{"extends":["./base.json","./future.json"]}"#,
    );
    let base = write(
        &root,
        "base.json",
        "\u{feff}{/* captured */\"compilerOptions\":{\"paths\":{\"@/*\":[\"old/*\"]}},}",
    );
    let snapshot = TsconfigOwnershipSnapshot::capture(&config).unwrap();
    write(
        &root,
        "future.json",
        r#"{"compilerOptions":{"paths":{"@/*":["future/*"]}}}"#,
    );
    let refreshed = TsconfigOwnershipSnapshot::capture(&config).unwrap();
    assert_eq!(
        refreshed.compiler_options(&config).unwrap().options["paths"]["@/*"][0],
        "future/*"
    );
    std::fs::remove_file(&config).unwrap();
    std::fs::remove_file(&base).unwrap();
    let options = snapshot.compiler_options(&config).unwrap();
    assert_eq!(options.options["paths"]["@/*"][0], "old/*");
    assert_eq!(options.unresolved_extends.len(), 1);
    assert_eq!(options.unresolved_extends[0].specifier, "./future.json");
    assert_eq!(options.unresolved_extends[0].config, config);
}

#[test]
fn ownership_snapshot_options_preserve_read_and_parse_failures_after_repair() {
    use crate::batch::error::CorsaError;
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().canonicalize().unwrap();
    for malformed in [false, true] {
        let config = root.join(if malformed {
            "malformed.json"
        } else {
            "absent.json"
        });
        if malformed {
            std::fs::write(&config, "{\n").unwrap();
        }
        let snapshot = TsconfigOwnershipSnapshot::capture(&config).unwrap();
        let original = snapshot.compiler_options(&config).unwrap_err();
        assert!(
            matches!(&original, CorsaError::Configuration(message) if message.starts_with(if malformed { "cannot parse tsconfig " } else { "cannot read tsconfig " }))
        );
        if malformed {
            assert!(
                original
                    .to_string()
                    .ends_with("EOF while parsing an object at line 2 column 0")
            );
        }
        std::fs::write(&config, r#"{"compilerOptions":{"strict":true}}"#).unwrap();
        let frozen = snapshot.compiler_options(&config).unwrap_err();
        assert!(matches!(frozen, CorsaError::Configuration(_)));
        assert_eq!(original.to_string(), frozen.to_string());
        assert_eq!(
            TsconfigOwnershipSnapshot::capture(&config)
                .unwrap()
                .compiler_options(&config)
                .unwrap()
                .options["strict"],
            true
        );
    }
    let captured = root.join("absent.json");
    let snapshot = TsconfigOwnershipSnapshot::capture(&captured).unwrap();
    let foreign = write(&root, "foreign.json", "{}");
    assert_eq!(
        snapshot.compiler_options(&foreign).unwrap_err().to_string(),
        format!(
            "Configuration error: uncaptured tsconfig project {}",
            foreign.display()
        )
    );
}

#[test]
fn frozen_options_reject_uncaptured_inputs_and_resolution_kinds() {
    use super::super::compiler_inputs::CompilerInputs;
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().canonicalize().unwrap();
    let config = write(&root, "tsconfig.json", r#"{"extends":"./base.json"}"#);
    let base = write(&root, "base.json", "{}");
    let inputs = CompilerInputs::default();
    inputs.read_jsonc(&config).unwrap();
    inputs.resolve("reference", &config, "./base.json", || Some(base.clone()));
    let (frozen, _) = inputs.freeze().unwrap();
    assert_eq!(
        frozen.compiler_options(&config).unwrap_err().to_string(),
        format!(
            "Configuration error: uncaptured tsconfig extends {:?} from {}",
            "./base.json",
            config.display()
        )
    );
    assert_eq!(
        frozen.compiler_options(&base).unwrap_err().to_string(),
        format!(
            "Configuration error: uncaptured tsconfig input {}",
            base.display()
        )
    );

    let inputs = CompilerInputs::default();
    inputs.read_jsonc(&config).unwrap();
    inputs.resolve("extends", &config, "./base.json", || Some(base.clone()));
    let (frozen, _) = inputs.freeze().unwrap();
    assert_eq!(
        frozen.compiler_options(&config).unwrap_err().to_string(),
        format!(
            "Configuration error: uncaptured tsconfig input {}",
            base.display()
        )
    );
}

#[cfg(unix)]
#[test]
fn ownership_snapshot_options_preserve_relative_symlink_extends() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().canonicalize().unwrap();
    let config = write(&root, "tsconfig.json", r#"{"extends":"./linked.json"}"#);
    let base = write(&root, "real/base.json", r#"{"extends":"./parent.json"}"#);
    write(
        &root,
        "real/parent.json",
        r#"{"compilerOptions":{"paths":{"@/*":["physical/*"]}}}"#,
    );
    let logical = write(
        &root,
        "parent.json",
        r#"{"compilerOptions":{"paths":{"@/*":["logical/*"]}}}"#,
    );
    let link = root.join("linked.json");
    std::os::unix::fs::symlink(&base, &link).unwrap();
    let snapshot = TsconfigOwnershipSnapshot::capture(&config).unwrap();
    let expected = vize_carton::typescript_project::CompilerOptionsContext::observed(&root)
        .load_flattened(
            Some(&config),
            &super::super::compiler_options::CanonConfigHost { inputs: None },
        )
        .unwrap();
    assert_eq!(snapshot.compiler_options(&config).unwrap(), expected);
    assert_eq!(expected.options["paths"]["@/*"][0], "logical/*");
    std::fs::remove_file(&link).unwrap();
    std::fs::remove_file(&base).unwrap();
    std::fs::remove_file(&logical).unwrap();
    assert!(snapshot.observations().needs_confirmation());
    assert_eq!(snapshot.compiler_options(&config).unwrap(), expected);
}

#[test]
fn ownership_snapshot_selects_membership_instead_of_the_first_reference() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().canonicalize().unwrap();
    let config = write(
        &root,
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"node.json"},{"path":"app.json"}]}"#,
    );
    let node = write(&root, "node.json", r#"{"include":["vite.config.ts"]}"#);
    let app = write(&root, "app.json", r#"{"include":["src/**/*"]}"#);
    let source = write(&root, "src/App.vue", "<template />");
    let script = write(&root, "src/plain.js", "export {};");
    let snapshot = TsconfigOwnershipSnapshot::capture(&config).unwrap();
    assert_eq!(
        snapshot.project_paths(),
        [config.clone(), node, app.clone()]
    );
    assert_eq!(snapshot.select(&source, TsconfigSourceKind::Typed), app);
    assert_eq!(
        snapshot.select(&script, TsconfigSourceKind::JavaScript),
        config
    );
    assert_eq!(
        snapshot.select(&root.join("other.vue"), TsconfigSourceKind::Typed),
        config
    );
    assert!(!snapshot.observations().needs_confirmation());

    write(&root, "app.json", r#"{"include":[]}"#);
    assert_eq!(snapshot.select(&source, TsconfigSourceKind::Typed), app);
    assert!(snapshot.observations().needs_confirmation());
    let refreshed = TsconfigOwnershipSnapshot::capture(&config).unwrap();
    assert_eq!(refreshed.select(&source, TsconfigSourceKind::Typed), config);

    write(&root, "app.json", r#"{"include":["src/**/*"]}"#);
    write(&root, "other.json", r#"{"include":["src/**/*"]}"#);
    write(
        &root,
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"app.json"},{"path":"other.json"}]}"#,
    );
    let ambiguous = TsconfigOwnershipSnapshot::capture(&config).unwrap();
    assert_eq!(ambiguous.select(&source, TsconfigSourceKind::Typed), config);
}

#[test]
fn ownership_snapshot_observes_inherited_specs_and_package_selection() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().canonicalize().unwrap();
    let config = write(
        &root,
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"app/tsconfig.json"}]}"#,
    );
    let app = write(
        &root,
        "app/tsconfig.json",
        "\u{feff}{\"extends\":\"@scope/config\",}",
    );
    let package = write(
        &root,
        "node_modules/@scope/config/package.json",
        r#"{"tsconfig":"selected.json"}"#,
    );
    let selected = write(
        &root,
        "node_modules/@scope/config/selected.json",
        r#"{"include":["../../../app/src/**/*"]}"#,
    );
    let source = write(&root, "app/src/App.vue", "<template />");
    let snapshot = TsconfigOwnershipSnapshot::capture(&config).unwrap();
    assert_eq!(snapshot.select(&source, TsconfigSourceKind::Typed), app);
    let observations = snapshot.observations().paths();
    assert!(
        observations.contains(&package),
        "winning package selection must be observed"
    );
    assert!(
        observations.contains(&selected),
        "inherited membership bytes must be observed"
    );
    let nearer = root.join("app/node_modules/@scope/config/package.json");
    assert!(
        observations.contains(&nearer),
        "nearer package absence must be observed"
    );
    assert!(!snapshot.observations().needs_confirmation());

    write(
        &root,
        "node_modules/@scope/config/package.json",
        r#"{"tsconfig":"other.json"}"#,
    );
    write(
        &root,
        "node_modules/@scope/config/other.json",
        r#"{"include":[]}"#,
    );
    assert!(snapshot.observations().needs_confirmation());
    let replaced = TsconfigOwnershipSnapshot::capture(&config).unwrap();
    assert_eq!(replaced.select(&source, TsconfigSourceKind::Typed), config);

    write(
        &root,
        "app/node_modules/@scope/config/package.json",
        r#"{"tsconfig":"nearer.json"}"#,
    );
    write(
        &root,
        "app/node_modules/@scope/config/nearer.json",
        r#"{"include":["../../../src/**/*"]}"#,
    );
    assert!(replaced.observations().needs_confirmation());
    let nearest = TsconfigOwnershipSnapshot::capture(&config).unwrap();
    assert_eq!(nearest.select(&source, TsconfigSourceKind::Typed), app);
}

#[test]
fn ownership_snapshot_keeps_legacy_invalid_spec_fallback_and_tracks_repairs() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().canonicalize().unwrap();
    let config = write(
        &root,
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"bad.json"}]}"#,
    );
    let bad = write(&root, "bad.json", "{");
    let source = write(&root, "src/App.vue", "<template />");
    let snapshot = TsconfigOwnershipSnapshot::capture(&config).unwrap();
    assert_eq!(snapshot.select(&source, TsconfigSourceKind::Typed), config);
    assert!(snapshot.observations().paths().contains(&bad));
    write(&root, "bad.json", r#"{"include":["src/**/*"]}"#);
    assert!(snapshot.observations().needs_confirmation());
    let repaired = TsconfigOwnershipSnapshot::capture(&config).unwrap();
    assert_eq!(repaired.select(&source, TsconfigSourceKind::Typed), bad);
    assert!(!repaired.observations().needs_confirmation());
}

#[test]
fn ownership_snapshot_retains_absent_reference_observations() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().canonicalize().unwrap();
    let config = write(
        &root,
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"future.json"},{"path":"app.json"}]}"#,
    );
    let app = write(&root, "app.json", r#"{"include":["src/**/*"]}"#);
    let source = write(&root, "src/App.vue", "<template />");
    let snapshot = TsconfigOwnershipSnapshot::capture(&config).unwrap();
    assert_eq!(snapshot.select(&source, TsconfigSourceKind::Typed), app);
    assert!(
        snapshot
            .observations()
            .paths()
            .contains(&root.join("future.json"))
    );
    write(&root, "future.json", r#"{"include":["src/**/*"]}"#);
    assert!(snapshot.observations().needs_confirmation());
    assert!(snapshot.observations().needs_confirmation());
    assert_eq!(snapshot.select(&source, TsconfigSourceKind::Typed), app);
    let refreshed = TsconfigOwnershipSnapshot::capture(&config).unwrap();
    assert_eq!(refreshed.select(&source, TsconfigSourceKind::Typed), config);
}

#[test]
fn ownership_snapshot_uses_the_host_case_policy_and_requires_absolute_roots() {
    fn require_send_sync<T: Send + Sync>() {}
    require_send_sync::<TsconfigOwnershipSnapshot>();
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().canonicalize().unwrap();
    let config = write(
        &root,
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"app.json"}]}"#,
    );
    let app = write(&root, "app.json", r#"{"include":["SRC/**/*"]}"#);
    let source = write(&root, "src/App.vue", "<template />");
    for (case_sensitive, expected) in [(true, &config), (false, &app)] {
        let snapshot = TsconfigOwnershipSnapshot::capture_with_options(
            &config,
            TsconfigOwnershipOptions::with_case_sensitive(case_sensitive),
        )
        .unwrap();
        assert_eq!(
            &snapshot.select(&source, TsconfigSourceKind::Typed),
            expected
        );
    }
    assert!(matches!(
        TsconfigOwnershipSnapshot::capture(Path::new("tsconfig.json")).err(),
        Some(crate::batch::error::CorsaError::Configuration(message))
            if message == "project ownership capture requires an absolute selected config path"
    ));
}
