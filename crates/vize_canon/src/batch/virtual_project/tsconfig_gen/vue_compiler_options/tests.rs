use super::super::super::VirtualProject;
use super::extends::check_unknown_props_enabled;
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use vize_carton::{ToCompactString, cstr};

fn unique_case_dir(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(cstr!("vue-compiler-options-{name}-{}", std::process::id()).as_str())
}

#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
fn bool_map(pairs: &[(&str, bool)]) -> Map<std::string::String, Value> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_owned(), Value::Bool(*value)))
        .collect()
}

#[test]
fn unknown_props_follow_vue_tsc_defaults() {
    assert!(!check_unknown_props_enabled(&Map::new()));
    assert!(check_unknown_props_enabled(&bool_map(&[(
        "strictTemplates",
        true
    )])));
    assert!(!check_unknown_props_enabled(&bool_map(&[
        ("strictTemplates", true),
        ("checkUnknownProps", false),
    ])));
    assert!(check_unknown_props_enabled(&bool_map(&[(
        "checkUnknownProps",
        true
    )])));
}

#[test]
fn tsconfig_without_vue_compiler_options_matches_vue_tsc() {
    let case_dir = unique_case_dir("default");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(&case_dir).unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{ "compilerOptions": { "strict": true } }"#,
    )
    .unwrap();
    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.set_tsconfig_path(Some(case_dir.join("tsconfig.json")));
    assert!(!project.checks_unknown_props());
    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn extended_strict_templates_enables_unknown_props() {
    let case_dir = unique_case_dir("extends");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(&case_dir).unwrap();
    fs::write(
        case_dir.join("tsconfig.base.json"),
        r#"{ "vueCompilerOptions": { "strictTemplates": true } }"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{ "extends": "./tsconfig.base.json", "compilerOptions": { "strict": true } }"#,
    )
    .unwrap();
    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.set_tsconfig_path(Some(case_dir.join("tsconfig.json")));
    assert!(project.checks_unknown_props());
    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn no_tsconfig_keeps_generated_virtual_ts_strict() {
    let case_dir = unique_case_dir("none");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(&case_dir).unwrap();
    let project = VirtualProject::new(&case_dir).unwrap();
    assert!(project.checks_unknown_props());
    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn implicit_project_loads_jsonc_vue_options_from_ordered_package_inheritance() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("node_modules/@vue/tsconfig")).unwrap();
    fs::write(
        root.path().join("node_modules/@vue/tsconfig/base.json"),
        r#"{
        // Existing vue-tsc settings are authoritative, including inherited flags.
        "compilerOptions": { "strict": true, "target": "ES2022" },
        "vueCompilerOptions": { "strictTemplates": true, "jsxSlots": true },
    }"#,
    )
    .unwrap();
    fs::write(
        root.path().join("overrides.json"),
        r#"{
        "vueCompilerOptions": { "checkUnknownProps": false }
    }"#,
    )
    .unwrap();
    fs::write(
        root.path().join("tsconfig.json"),
        r#"{
        "extends": ["@vue/tsconfig/base.json", "./overrides.json"],
        "vueCompilerOptions": { "checkUnknownComponents": false },
    }"#,
    )
    .unwrap();
    let mut project = VirtualProject::new(root.path()).unwrap();
    let checks = project.virtual_ts_check_options;
    assert!(!checks.check_unknown_props);
    assert!(!checks.check_unknown_components);
    assert!(checks.check_unknown_directives);
    assert!(checks.check_unknown_events);
    assert!(checks.jsx_slots);
    let compiler = project
        .load_compiler_options(Some(&root.path().join("tsconfig.json")))
        .unwrap();
    assert_eq!(compiler["strict"], Value::Bool(true));
    assert_eq!(compiler["target"], serde_json::json!("ES2022"));
    fs::write(
        root.path().join("overrides.json"),
        r#"{
        "vueCompilerOptions": { "checkUnknownProps": true }
    }"#,
    )
    .unwrap();
    project.refresh_compiler_configuration();
    assert!(project.virtual_ts_check_options.check_unknown_props);
}

#[test]
fn vue_options_follow_the_selected_reference_without_leaking_from_siblings() {
    let root = tempfile::tempdir().unwrap();
    for name in ["app", "server"] {
        fs::create_dir(root.path().join(name)).unwrap();
    }
    fs::write(
        root.path().join("tsconfig.json"),
        r#"{
        "files": [], "references": [{"path":"./app"},{"path":"./server"}]
    }"#,
    )
    .unwrap();
    fs::write(
        root.path().join("app/tsconfig.json"),
        r#"{
        "include": ["**/*.vue"], "vueCompilerOptions": {"strictTemplates":true,"target":2.7}
    }"#,
    )
    .unwrap();
    fs::write(
        root.path().join("server/tsconfig.json"),
        r#"{
        "include": ["**/*.ts"], "vueCompilerOptions": {"strictTemplates":false,"target":3.5}
    }"#,
    )
    .unwrap();
    let app = root.path().join("app/App.vue");
    fs::write(&app, "<template><MissingWidget /></template>").unwrap();
    let mut project = VirtualProject::new(root.path()).unwrap();
    project.use_effective_tsconfig_for_source(&app);
    assert!(project.virtual_ts_check_options.check_unknown_components);
    project.register_path(&app).unwrap();
    assert!(
        project
            .find_by_original(&app)
            .unwrap()
            .content
            .contains("__vize_global_component_")
    );
    assert_eq!(
        super::snapshot_tsconfig_vue_target(
            root.path(),
            &project.effective_tsconfig_path().unwrap()
        )
        .unwrap(),
        Some(vize_carton::config::VueVersion::V2_7)
    );
}

#[test]
fn vue_target_uses_inheritance_and_remains_distinct_from_typescript_target() {
    use vize_carton::config::VueVersion;
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("tsconfig.json");
    fs::write(
        root.path().join("base.json"),
        r#"{
        "compilerOptions": {"target":"ES2022"}, "vueCompilerOptions":{"target":2.7}
    }"#,
    )
    .unwrap();
    fs::write(&config, r#"{"extends":"./base.json"}"#).unwrap();
    assert_eq!(
        super::snapshot_tsconfig_vue_target(root.path(), &config).unwrap(),
        Some(VueVersion::V2_7)
    );
    for (target, expected) in [
        (2.0, VueVersion::V2),
        (2.6, VueVersion::V2),
        (2.7, VueVersion::V2_7),
        (3.0, VueVersion::V3),
        (3.5, VueVersion::V3),
        (99.0, VueVersion::V3),
    ] {
        fs::write(
            &config,
            serde_json::json!({"extends":"./base.json", "vueCompilerOptions":{"target":target}})
                .to_compact_string(),
        )
        .unwrap();
        assert_eq!(
            super::snapshot_tsconfig_vue_target(root.path(), &config).unwrap(),
            Some(expected)
        );
    }
    for target in [
        serde_json::json!(2.8),
        serde_json::json!(4),
        serde_json::json!("2.7"),
        Value::Null,
    ] {
        fs::write(
            &config,
            serde_json::json!({"vueCompilerOptions":{"target":target}}).to_compact_string(),
        )
        .unwrap();
        let error = super::snapshot_tsconfig_vue_target(root.path(), &config).unwrap_err();
        assert!(cstr!("{error}").contains("vueCompilerOptions.target"));
    }
    fs::write(&config, r#"{"compilerOptions":{"target":"ES2022"}}"#).unwrap();
    assert_eq!(
        super::snapshot_tsconfig_vue_target(root.path(), &config).unwrap(),
        None
    );
}

#[test]
fn auto_target_uses_the_project_local_vue_manifest_and_reports_unresolved_versions() {
    use vize_carton::config::VueVersion;
    let root = tempfile::tempdir().unwrap();
    let project = root.path().join("apps/web");
    fs::create_dir_all(project.join("node_modules/vue")).unwrap();
    fs::create_dir_all(root.path().join("node_modules/vue")).unwrap();
    fs::write(
        root.path().join("node_modules/vue/package.json"),
        r#"{"version":"3.5.0"}"#,
    )
    .unwrap();
    let config = project.join("tsconfig.json");
    let manifest = project.join("node_modules/vue/package.json");
    fs::write(&config, r#"{"vueCompilerOptions":{"target":"auto"}}"#).unwrap();
    for (version, expected) in [
        ("2.6.14", VueVersion::V2),
        ("2.7.16", VueVersion::V2_7),
        ("3.6.0-beta.1", VueVersion::V3),
    ] {
        fs::write(
            &manifest,
            serde_json::json!({"version":version}).to_compact_string(),
        )
        .unwrap();
        assert_eq!(
            super::snapshot_tsconfig_vue_target(root.path(), &config).unwrap(),
            Some(expected)
        );
    }
    fs::write(&manifest, r#"{"version":"4.0.0"}"#).unwrap();
    assert!(
        super::snapshot_tsconfig_vue_target(root.path(), &config)
            .unwrap_err()
            .to_compact_string()
            .contains("unsupported installed Vue version")
    );
    fs::remove_dir_all(project.join("node_modules")).unwrap();
    fs::remove_dir_all(root.path().join("node_modules")).unwrap();
    assert!(
        super::snapshot_tsconfig_vue_target(root.path(), &config)
            .unwrap_err()
            .to_compact_string()
            .contains("cannot resolve installed Vue")
    );
}

#[test]
fn governing_paths_include_inherited_configs_and_auto_target_vue_manifest() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("node_modules/vue")).unwrap();
    let manifest = root.path().join("node_modules/vue/package.json");
    fs::write(&manifest, r#"{"version":"2.7.16"}"#).unwrap();
    let base = root.path().join("base.json");
    fs::write(&base, r#"{"vueCompilerOptions":{"target":"auto"}}"#).unwrap();
    let config = root.path().join("tsconfig.json");
    fs::write(&config, r#"{"extends":"./base.json"}"#).unwrap();
    let project = VirtualProject::new(root.path()).unwrap();
    let paths = project.governing_config_paths();
    for path in [config, base, manifest] {
        let path = vize_carton::path::canonicalize_non_verbatim(&path);
        assert!(paths.contains(&path), "missing {path:?} from {paths:?}");
    }
}
