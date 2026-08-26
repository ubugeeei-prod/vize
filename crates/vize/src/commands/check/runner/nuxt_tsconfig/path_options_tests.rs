use std::{fs, path::Path};

use super::write_nuxt_fallback_tsconfig_in_cache;
use crate::commands::check::nuxt::NuxtPathAlias;

#[test]
fn effective_base_url_anchors_paths_declared_by_a_child_config() {
    let case = tempfile::tempdir().unwrap();
    write(
        case.path(),
        "base.json",
        r#"{ "compilerOptions": { "baseUrl": "./src" } }"#,
    );
    write(
        case.path(),
        "tsconfig.json",
        r##"{
  "extends": "./base.json",
  "compilerOptions": { "paths": { "~/*": ["shared/*"] } }
}"##,
    );

    let paths = prepared_paths(case.path(), "tsconfig.json");
    assert_eq!(
        paths["~/*"],
        serde_json::json!([target(case.path(), "src/shared/*")])
    );
}

#[test]
fn child_base_url_reanchors_paths_declared_by_a_parent_config() {
    let case = tempfile::tempdir().unwrap();
    write(
        case.path(),
        "base.json",
        r##"{ "compilerOptions": { "paths": { "~/*": ["shared/*"] } } }"##,
    );
    write(
        case.path(),
        "tsconfig.json",
        r#"{
  "extends": "./base.json",
  "compilerOptions": { "baseUrl": "./src" }
}"#,
    );

    let paths = prepared_paths(case.path(), "tsconfig.json");
    assert_eq!(
        paths["~/*"],
        serde_json::json!([target(case.path(), "src/shared/*")])
    );
}

#[test]
fn later_extends_paths_keep_the_effective_base_url_from_an_earlier_entry() {
    let case = tempfile::tempdir().unwrap();
    write(
        case.path(),
        "base-url.json",
        r#"{ "compilerOptions": { "baseUrl": "./source" } }"#,
    );
    write(
        case.path(),
        "paths.json",
        r##"{ "compilerOptions": { "paths": { "~/*": ["from-paths/*"] } } }"##,
    );
    write(
        case.path(),
        "tsconfig.json",
        r#"{ "extends": ["./base-url.json", "./paths.json"] }"#,
    );

    let paths = prepared_paths(case.path(), "tsconfig.json");
    assert_eq!(
        paths["~/*"],
        serde_json::json!([target(case.path(), "source/from-paths/*")])
    );
}

#[test]
fn later_explicit_empty_paths_clear_an_earlier_extends_entry() {
    let case = tempfile::tempdir().unwrap();
    write(
        case.path(),
        "with-paths.json",
        r##"{ "compilerOptions": { "paths": { "old/*": ["old/*"] } } }"##,
    );
    write(
        case.path(),
        "without-paths.json",
        r#"{ "compilerOptions": { "paths": {} } }"#,
    );
    write(
        case.path(),
        "tsconfig.json",
        r#"{ "extends": ["./with-paths.json", "./without-paths.json"] }"#,
    );

    let paths = prepared_paths(case.path(), "tsconfig.json");
    assert!(!paths.contains_key("old/*"));
    assert!(paths.contains_key("#fallback/*"));
}

#[test]
fn published_snapshot_never_rereads_a_mutated_authored_config() {
    let case = tempfile::tempdir().unwrap();
    let tsconfig = case.path().join("tsconfig.json");
    write(
        case.path(),
        "tsconfig.json",
        r#"{ "compilerOptions": { "strict": true } }"#,
    );
    let first = prepare(case.path(), &tsconfig);
    let first_path = first.path().unwrap().to_path_buf();
    let first_bytes = fs::read(&first_path).unwrap();

    write(
        case.path(),
        "tsconfig.json",
        r#"{ "compilerOptions": { "strict": false } }"#,
    );
    let second = prepare(case.path(), &tsconfig);
    let second_path = second.path().unwrap().to_path_buf();
    let second_value: serde_json::Value =
        serde_json::from_slice(&fs::read(&second_path).unwrap()).unwrap();

    assert_ne!(first_path, second_path);
    assert_eq!(fs::read(first_path).unwrap(), first_bytes);
    assert!(second_value.get("extends").is_none());
    assert_eq!(
        second_value["compilerOptions"]["strict"],
        serde_json::json!(false)
    );
}

#[test]
fn authored_paths_override_inherited_and_nuxt_only_fills_missing_aliases() {
    let case = tempfile::tempdir().unwrap();
    write(
        case.path(),
        "base.json",
        r##"{ "compilerOptions": { "paths": { "#base/*": ["types/*"] } } }"##,
    );
    write(
        case.path(),
        "tsconfig.json",
        r##"{
  "extends": "./base.json",
  "compilerOptions": { "paths": { "~/*": ["custom/*"] } }
}"##,
    );
    let config = case.path().join("tsconfig.json");
    let prepared = write_nuxt_fallback_tsconfig_in_cache(
        Some(&config),
        case.path(),
        case.path(),
        &[
            NuxtPathAlias {
                pattern: "~/*".into(),
                targets: vec!["app/*".into()],
            },
            NuxtPathAlias {
                pattern: "#shared/*".into(),
                targets: vec!["shared/*".into()],
            },
        ],
        &case.path().join("cache"),
    )
    .unwrap();
    let value: serde_json::Value =
        serde_json::from_slice(&fs::read(prepared.path().unwrap()).unwrap()).unwrap();
    let paths = value["compilerOptions"]["paths"].as_object().unwrap();

    assert!(value.get("extends").is_none());
    assert!(!paths.contains_key("#base/*"));
    assert_eq!(
        paths["~/*"],
        serde_json::json!([target(case.path(), "custom/*")])
    );
    assert_eq!(
        paths["#shared/*"],
        serde_json::json!([physical_target(case.path(), "shared/*")])
    );
}

#[test]
fn invalid_path_options_survive_for_the_authored_option_probe() {
    let case = tempfile::tempdir().unwrap();
    write(
        case.path(),
        "tsconfig.json",
        r#"{ "compilerOptions": { "baseUrl": 42, "paths": "invalid" } }"#,
    );

    let prepared = prepare(case.path(), &case.path().join("tsconfig.json"));
    let value: serde_json::Value =
        serde_json::from_slice(&fs::read(prepared.path().unwrap()).unwrap()).unwrap();
    assert_eq!(value["compilerOptions"]["baseUrl"], serde_json::json!(42));
    assert_eq!(
        value["compilerOptions"]["paths"],
        serde_json::json!("invalid")
    );
    assert!(
        value["compilerOptions"]["paths"]
            .get("#fallback/*")
            .is_none(),
        "an invalid authored paths value cannot receive a generated alias"
    );
}

fn prepared_paths(root: &Path, config: &str) -> serde_json::Map<String, serde_json::Value> {
    let config = root.join(config);
    let prepared = prepare(root, &config);
    let value: serde_json::Value =
        serde_json::from_slice(&fs::read(prepared.path().unwrap()).unwrap()).unwrap();
    value["compilerOptions"]["paths"]
        .as_object()
        .unwrap()
        .clone()
}

fn prepare(root: &Path, config: &Path) -> super::PreparedCheckerTsconfig {
    write_nuxt_fallback_tsconfig_in_cache(
        Some(config),
        root,
        root,
        &[NuxtPathAlias {
            pattern: "#fallback/*".into(),
            targets: vec!["fallback/*".into()],
        }],
        &root.join("cache"),
    )
    .unwrap()
}

#[test]
fn dependency_tree_components_are_rejected_case_insensitively() {
    let case = tempfile::tempdir().unwrap();
    let project = case.path().join("project");
    let cache = case.path().join("NODE_MODULES/vize/check/nuxt");
    assert!(super::validate_config_cache_root(&cache, &project).is_err());
}

fn target(root: &Path, relative: &str) -> String {
    root.join(relative)
        .to_string_lossy()
        .replace('\\', "/")
        .into()
}

fn physical_target(root: &Path, relative: &str) -> String {
    vize_s0::path::canonicalize_non_verbatim(root)
        .join(relative)
        .to_string_lossy()
        .replace('\\', "/")
        .into()
}

fn write(root: &Path, relative: &str, content: &str) {
    fs::write(root.join(relative), content).unwrap();
}
