use std::path::{Path, PathBuf};

use super::EditorProjectConfiguration;
use crate::corsa_bridge::{CorsaBridge, CorsaBridgeConfig};

#[test]
fn inherited_nuxt_membership_retains_own_hidden_declarations_and_refuses_siblings() {
    let workspace = tempfile::tempdir().unwrap();
    let app = workspace.path().join("apps/volt");
    let host = write(&app.join("src/FilesCard.vue"), "<template />");
    write(
        &app.join("tsconfig.json"),
        r#"{"extends":"./.nuxt/tsconfig.json"}"#,
    );
    write(
        &app.join(".nuxt/tsconfig.json"),
        r#"{"include":["../src/**/*.vue","./imports.d.ts"],"exclude":["../src/ignored.d.ts"]}"#,
    );
    let own = write(&app.join(".nuxt/imports.d.ts"), "export {};");
    let sibling = write(
        &workspace.path().join("apps/sibling/.nuxt/imports.d.ts"),
        "export {};",
    );
    let excluded = write(&app.join("src/ignored.d.ts"), "export {};");
    let configuration = EditorProjectConfiguration::for_source(&host, Some(workspace.path()), None);
    assert_eq!(configuration.root, physical(workspace.path()));
    assert_eq!(
        configuration.tsconfig,
        Some(physical(&app.join("tsconfig.json")))
    );
    assert_eq!(
        configuration.reference_paths(&[sibling, excluded, own.clone(), own.clone()]),
        vec![own]
    );
}

#[test]
fn solution_shell_declarations_follow_the_actual_unique_source_owner() {
    let workspace = tempfile::tempdir().unwrap();
    let shell = write(
        &workspace.path().join("tsconfig.json"),
        r#"{"files":[],"references":[{"path":"apps/volt"},{"path":"apps/sibling"}]}"#,
    );
    let app = workspace.path().join("apps/volt");
    let sibling = workspace.path().join("apps/sibling");
    let host = write(&app.join("src/FilesCard.vue"), "<template />");
    for root in [&app, &sibling] {
        write(
            &root.join("tsconfig.json"),
            r#"{"include":["src/**/*.vue",".nuxt/imports.d.ts"]}"#,
        );
    }
    let own = write(&app.join(".nuxt/imports.d.ts"), "export {};");
    let foreign = write(&sibling.join(".nuxt/imports.d.ts"), "export {};");
    let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
        working_dir: Some(workspace.path().into()),
        tsconfig_path: Some(shell),
        ..Default::default()
    });
    assert_eq!(
        bridge.scoped_vue_reference_paths(&host, &[foreign, own.clone()]),
        vec![own]
    );
}

#[test]
fn explicit_relative_configuration_uses_the_bridge_root_instead_of_nearest_config() {
    let workspace = tempfile::tempdir().unwrap();
    let app = workspace.path().join("app");
    let host = write(&app.join("src/Host.vue"), "<template />");
    write(&app.join("tsconfig.json"), r#"{"files":[]}"#);
    write(
        &workspace.path().join("configs/editor.json"),
        r#"{"include":["../app/src/**/*.vue","../app/generated/ambient.d.ts"]}"#,
    );
    let own = write(&app.join("generated/ambient.d.ts"), "export {};");
    let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
        working_dir: Some(workspace.path().into()),
        tsconfig_path: Some("configs/editor.json".into()),
        ..Default::default()
    });
    assert_eq!(
        bridge.scoped_vue_reference_paths(&host, &[own.clone()]),
        vec![own]
    );
}

#[test]
fn a_configured_project_without_readable_membership_never_admits_workspace_globals() {
    let workspace = tempfile::tempdir().unwrap();
    let host = write(&workspace.path().join("Host.vue"), "<template />");
    let ambient = write(&workspace.path().join("ambient.d.ts"), "export {};");
    let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
        working_dir: Some(workspace.path().into()),
        tsconfig_path: Some(workspace.path().join("missing.json")),
        ..Default::default()
    });
    assert!(
        bridge
            .scoped_vue_reference_paths(&host, &[ambient])
            .is_empty()
    );
}

#[test]
fn no_config_keeps_only_physical_existing_workspace_declarations() {
    let workspace = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let host = write(&workspace.path().join("Host.vue"), "<template />");
    let dts = write(&workspace.path().join("globals.d.ts"), "export {};");
    let dmts = write(&workspace.path().join("globals.d.mts"), "export {};");
    let dcts = write(&workspace.path().join("globals.d.cts"), "export {};");
    let script = write(&workspace.path().join("runtime.ts"), "export {};");
    let foreign = write(&outside.path().join("globals.d.ts"), "export {};");
    let missing = workspace.path().join("missing.d.ts");
    let configuration = EditorProjectConfiguration::for_source(&host, Some(workspace.path()), None);
    let mut expected = vec![dts.clone(), dmts.clone(), dcts.clone()];
    expected.sort();
    assert_eq!(
        configuration.reference_paths(&[dts, dmts, dcts, script, foreign, missing]),
        expected
    );
}

#[test]
fn changed_configuration_is_reread_without_reusing_old_declaration_membership() {
    let workspace = tempfile::tempdir().unwrap();
    let host = write(&workspace.path().join("Host.vue"), "<template />");
    let ambient = write(&workspace.path().join("globals.d.ts"), "export {};");
    let config = workspace.path().join("tsconfig.json");
    write(&config, r#"{"files":["Host.vue","globals.d.ts"]}"#);
    let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
        working_dir: Some(workspace.path().into()),
        ..Default::default()
    });
    assert_eq!(
        bridge.scoped_vue_reference_paths(&host, &[ambient.clone()]),
        vec![ambient.clone()]
    );
    write(&config, r#"{"files":["Host.vue"]}"#);
    assert!(
        bridge
            .scoped_vue_reference_paths(&host, &[ambient])
            .is_empty()
    );
}

#[cfg(unix)]
#[test]
fn logical_workspace_spellings_keep_one_physical_config_and_declaration_scope() {
    let workspace = tempfile::tempdir().unwrap();
    let physical_root = workspace.path().join("physical");
    let logical = workspace.path().join("logical");
    let host = write(&physical_root.join("src/Host.vue"), "<template />");
    write(
        &physical_root.join("tsconfig.json"),
        r#"{"include":["src/**/*.vue",".nuxt/imports.d.ts"]}"#,
    );
    let ambient = write(&physical_root.join(".nuxt/imports.d.ts"), "export {};");
    std::os::unix::fs::symlink(&physical_root, &logical).unwrap();
    let configuration =
        EditorProjectConfiguration::for_source(&logical.join("src/Host.vue"), Some(&logical), None);
    assert_eq!(
        configuration.tsconfig,
        Some(physical(&physical_root.join("tsconfig.json")))
    );
    assert_eq!(
        configuration.reference_paths(&[logical.join(".nuxt/imports.d.ts")]),
        vec![ambient]
    );
    assert_eq!(physical(&logical.join("src/Host.vue")), host);
}

fn write(path: &Path, contents: &str) -> PathBuf {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, contents).unwrap();
    physical(path)
}

fn physical(path: &Path) -> PathBuf {
    vize_carton::path::canonicalize_non_verbatim(path)
}
