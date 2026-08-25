//! CLI integration regressions for Canon's shared referenced-project owner.

#![allow(clippy::disallowed_macros, clippy::disallowed_types)]

use std::path::{Path, PathBuf};

use vize_s0::path::canonicalize_non_verbatim;

use super::{TsconfigInputCache, resolve_tsconfig_for_files};

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn owner(root: &Path, file: PathBuf) -> PathBuf {
    resolve_tsconfig_for_files(
        Some(&root.join("tsconfig.json")),
        &[file],
        false,
        &mut TsconfigInputCache::default(),
    )
    .unwrap()
}

#[test]
fn inherited_out_dir_does_not_steal_generated_project_input() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"./app.json"},{"path":"./generated.json"}]}"#,
    );
    write(
        root.path(),
        "base.json",
        r#"{"compilerOptions":{"outDir":"dist"}}"#,
    );
    write(root.path(), "app.json", r#"{"extends":"./base.json"}"#);
    write(
        root.path(),
        "generated.json",
        r#"{"include":["dist/**/*"]}"#,
    );
    write(root.path(), "dist/App.vue", "<template />");

    assert_eq!(
        owner(root.path(), root.path().join("dist/App.vue")),
        canonicalize_non_verbatim(&root.path().join("generated.json"))
    );
}

#[test]
fn transitive_reference_leaf_owns_cli_input() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"./workspace.json"}]}"#,
    );
    write(
        root.path(),
        "workspace.json",
        r#"{"files":[],"references":[{"path":"./app.json"}]}"#,
    );
    write(root.path(), "app.json", r#"{"include":["src/**/*"]}"#);
    write(root.path(), "src/App.vue", "<template />");

    assert_eq!(
        owner(root.path(), root.path().join("src/App.vue")),
        canonicalize_non_verbatim(&root.path().join("app.json"))
    );
}

#[test]
fn overlapping_referenced_projects_fail_closed_to_solution_shell() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"./a.json"},{"path":"./b.json"}]}"#,
    );
    write(root.path(), "a.json", r#"{"include":["src/**/*"]}"#);
    write(root.path(), "b.json", r#"{"include":["src/**/*"]}"#);
    write(root.path(), "src/App.vue", "<template />");

    assert_eq!(
        owner(root.path(), root.path().join("src/App.vue")),
        canonicalize_non_verbatim(&root.path().join("tsconfig.json"))
    );
}
