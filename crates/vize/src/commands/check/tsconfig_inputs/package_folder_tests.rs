//! Tests for TypeScript's implicit exclusion of `node_modules`,
//! `bower_components` and `jspm_packages` from wildcard `include` segments
//! (#3385).
//!
//! Every expectation here was taken from `tsgo -p tsconfig.json --listFiles` on
//! the same tree; the case names say which behavior each one pins.
//!
//! The case directories deliberately live under [`std::env::temp_dir`] rather
//! than the crate's `target/`: the collector walks with `ignore`'s standard
//! filters, so inside the repository the checkout's `.gitignore` would hide
//! `node_modules` on its own and the assertions would pass without the fix.

#![allow(clippy::disallowed_macros, clippy::disallowed_types)]

use std::path::{Path, PathBuf};

use vize_s0::cstr;

use super::TsconfigInputCache;

fn collect_default_check_files(project_root: &Path, tsconfig_path: &Path) -> Vec<PathBuf> {
    super::collect_default_check_files(
        project_root,
        Some(tsconfig_path),
        false,
        &mut TsconfigInputCache::default(),
    )
}

fn write(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, contents).unwrap();
}

/// A fresh case directory outside the repository, plus the workspace every case
/// in this module shares: one real source file per package folder name, nested a
/// package deep, and one ordinary source file beside them.
fn workspace(name: &str, tsconfig: &str) -> PathBuf {
    static NEXT_CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let root = std::env::temp_dir().join(
        cstr!(
            "vize-package-folders-{name}-{}-{case_id}",
            std::process::id()
        )
        .as_str(),
    );
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();

    write(&root, "packages/a/index.ts", "export const a = 1;\n");
    write(&root, "packages/a/sub/deep.ts", "export const s = 1;\n");
    write(
        &root,
        "packages/a/node_modules/dep/index.ts",
        "export const d = 1;\n",
    );
    write(
        &root,
        "packages/a/bower_components/dep/index.ts",
        "export const b = 1;\n",
    );
    write(
        &root,
        "packages/a/jspm_packages/dep/index.ts",
        "export const j = 1;\n",
    );
    write(&root, "node_modules/top/index.ts", "export const t = 1;\n");
    write(&root, "tsconfig.json", tsconfig);

    // `temp_dir` is a symlink on macOS, and the collector returns canonicalized
    // paths, so the case root has to be canonical for `relative_paths`.
    vize_s0::path::canonicalize_non_verbatim(&root)
}

fn collected(root: &Path) -> Vec<String> {
    collect_default_check_files(root, &root.join("tsconfig.json"))
        .iter()
        .map(|path| {
            path.strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect()
}

#[test]
fn wildcard_include_skips_package_folders_nested_below_the_tsconfig_directory() {
    // tsgo, same tree, `include: ["packages/**/*.ts"]`:
    //   packages/a/index.ts
    //   packages/a/sub/deep.ts
    // Anchoring the default `exclude` at the tsconfig directory only, as vize
    // did, let all three nested package folders in as program roots.
    let root = workspace("nested", r#"{ "include": ["packages/**/*.ts"] }"#);

    assert_eq!(
        collected(&root),
        vec!["packages/a/index.ts", "packages/a/sub/deep.ts"]
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn an_explicit_empty_exclude_does_not_re_enable_nested_package_folders() {
    // The exclusion is part of wildcard expansion, not of `exclude`: tsgo lists
    // the same two files with `"exclude": []` as without it.
    let root = workspace(
        "empty-exclude",
        r#"{ "include": ["packages/**/*.ts"], "exclude": [] }"#,
    );

    assert_eq!(
        collected(&root),
        vec!["packages/a/index.ts", "packages/a/sub/deep.ts"]
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_user_exclude_still_narrows_the_wildcard_it_is_given() {
    // The new filter must not swallow the user's own `exclude`: tsgo lists only
    // `packages/a/index.ts` here.
    let root = workspace(
        "user-exclude",
        r#"{ "include": ["packages/**/*.ts"], "exclude": ["packages/a/sub"] }"#,
    );

    assert_eq!(collected(&root), vec!["packages/a/index.ts"]);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn an_include_entry_that_names_a_nested_package_folder_file_literally_is_kept() {
    // A spec with no wildcards is matched literally, so tsgo lists
    // `packages/a/node_modules/dep/index.ts` alongside the wildcard's files.
    let root = workspace(
        "literal-file",
        r#"{ "include": ["packages/**/*.ts", "packages/a/node_modules/dep/index.ts"] }"#,
    );

    assert_eq!(
        collected(&root),
        vec![
            "packages/a/index.ts",
            "packages/a/node_modules/dep/index.ts",
            "packages/a/sub/deep.ts",
        ]
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_literal_package_folder_segment_after_a_wildcard_segment_is_kept() {
    // `packages/*/node_modules/dep/*.ts`: only `*` segments carry the implicit
    // exclusion, so the literal `node_modules` segment resolves normally and
    // tsgo lists exactly this one file.
    let root = workspace(
        "literal-segment",
        r#"{ "include": ["packages/*/node_modules/dep/*.ts"] }"#,
    );

    assert_eq!(
        collected(&root),
        vec!["packages/a/node_modules/dep/index.ts"]
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_recursive_wildcard_below_a_literal_package_folder_segment_stays_shallow() {
    // `node_modules/**/*.ts` keeps its literal first segment but the `**` still
    // rejects a package folder at the depths it consumes, so tsgo lists
    // `node_modules/top/index.ts` and nothing from a deeper `node_modules`.
    let root = workspace(
        "literal-root-segment",
        r#"{ "include": ["node_modules/**/*.ts"] }"#,
    );
    write(
        &root,
        "node_modules/top/node_modules/inner/index.ts",
        "export const i = 1;\n",
    );

    assert_eq!(collected(&root), vec!["node_modules/top/index.ts"]);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_literal_package_folder_at_the_include_root_is_a_program_root() {
    // The reachability condition of #3395: vendored-dependency and
    // "typecheck one dependency" setups name the package folder in `include`.
    // tsgo, same tree:
    //   node_modules/top/index.ts
    // Vize's anchored default `exclude` used to shadow that literal segment and
    // collect nothing at all.
    let root = workspace("literal-root", r#"{ "include": ["node_modules/*/*.ts"] }"#);

    assert_eq!(collected(&root), vec!["node_modules/top/index.ts"]);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_other_two_package_folder_names_are_program_roots_when_named_literally() {
    // `bower_components` and `jspm_packages` get the same treatment as
    // `node_modules`: implicitly excluded from a wildcard segment, honored as a
    // literal one. tsgo lists both of these files.
    let root = workspace(
        "literal-root-others",
        r#"{ "include": ["bower_components/**/*.ts", "jspm_packages/**/*.ts"] }"#,
    );
    write(
        &root,
        "bower_components/b/index.ts",
        "export const b = 1;\n",
    );
    write(&root, "jspm_packages/j/index.ts", "export const j = 1;\n");

    assert_eq!(
        collected(&root),
        vec!["bower_components/b/index.ts", "jspm_packages/j/index.ts"]
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_files_entry_inside_a_nested_package_folder_is_kept() {
    // `files` bypasses include expansion entirely in tsc, and tsgo lists the
    // named file.
    let root = workspace(
        "files-entry",
        r#"{ "files": ["packages/a/node_modules/dep/index.ts"] }"#,
    );

    assert_eq!(
        collected(&root),
        vec!["packages/a/node_modules/dep/index.ts"]
    );

    let _ = std::fs::remove_dir_all(&root);
}
