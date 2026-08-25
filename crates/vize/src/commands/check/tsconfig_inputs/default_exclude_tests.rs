//! Tests for `tsc`'s real default `exclude` — `outDir` and `declarationDir`,
//! and nothing else (#3395).
//!
//! Every expectation here was taken from `tsgo -p tsconfig.json --noEmit
//! --listFiles` on the same tree. The three package folder names are *not* part
//! of this default: they are rejected from wildcard `include` segments instead,
//! which `package_folder_tests` covers.
//!
//! Case directories live under [`std::env::temp_dir`] for the reason spelled out
//! in `package_folder_tests`: inside the checkout, `.gitignore` would hide
//! `node_modules` on its own and hide the behavior under test.

#![allow(clippy::disallowed_macros, clippy::disallowed_types)]

use std::path::{Path, PathBuf};

use vize_s0::cstr;

use super::TsconfigInputCache;

fn write(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, contents).unwrap();
}

/// A fresh case directory outside the repository holding one source file in
/// each of the three directories these cases care about, plus a dependency copy
/// that no case may ever collect.
fn workspace(name: &str) -> PathBuf {
    static NEXT_CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let root = std::env::temp_dir().join(
        cstr!(
            "vize-default-exclude-{name}-{}-{case_id}",
            std::process::id()
        )
        .as_str(),
    );
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();

    write(&root, "src/index.ts", "export const s = 1;\n");
    write(&root, "dist/built.ts", "export const d = 1;\n");
    write(&root, "types/built.ts", "export const y = 1;\n");
    write(&root, "node_modules/dep/index.ts", "export const p = 1;\n");

    // `temp_dir` is a symlink on macOS and the collector returns canonicalized
    // paths, so the case root has to be canonical for `collected`.
    vize_s0::path::canonicalize_non_verbatim(&root)
}

fn collected(root: &Path, tsconfig: &str) -> Vec<String> {
    write(root, "tsconfig.json", tsconfig);
    super::collect_default_check_files(
        root,
        Some(&root.join("tsconfig.json")),
        false,
        &mut TsconfigInputCache::default(),
    )
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
fn out_dir_is_excluded_when_the_config_declares_no_exclude() {
    // tsgo: src/index.ts, types/built.ts.
    let root = workspace("out-dir");

    assert_eq!(
        collected(&root, r#"{ "compilerOptions": { "outDir": "dist" } }"#),
        vec!["src/index.ts", "types/built.ts"]
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn declaration_dir_is_excluded_when_the_config_declares_no_exclude() {
    // tsgo: dist/built.ts, src/index.ts. `declarationDir` is excluded even
    // though tsc also reports TS5069 for it without `declaration`.
    let root = workspace("declaration-dir");

    assert_eq!(
        collected(
            &root,
            r#"{ "compilerOptions": { "declarationDir": "types" } }"#
        ),
        vec!["dist/built.ts", "src/index.ts"]
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn nothing_is_excluded_by_default_without_out_dir_or_declaration_dir() {
    // tsgo: dist/built.ts, src/index.ts, types/built.ts — and never the
    // dependency copy, which the default `**/*` include rejects as a wildcard
    // segment rather than as an `exclude` entry.
    let root = workspace("no-output-dirs");

    assert_eq!(
        collected(&root, r#"{ "compilerOptions": { "strict": true } }"#),
        vec!["dist/built.ts", "src/index.ts", "types/built.ts"]
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn an_explicit_empty_exclude_replaces_the_out_dir_default() {
    // The default is the `exclude` *field*'s default, so spelling out any
    // `exclude` drops it: tsgo lists the output directory too.
    let root = workspace("empty-exclude");

    assert_eq!(
        collected(
            &root,
            r#"{ "compilerOptions": { "outDir": "dist" }, "exclude": [] }"#
        ),
        vec!["dist/built.ts", "src/index.ts", "types/built.ts"]
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn an_unrelated_explicit_exclude_replaces_the_out_dir_default() {
    // Same rule with a non-empty `exclude`: tsgo lists dist/built.ts and
    // types/built.ts, and drops only what the user named.
    let root = workspace("unrelated-exclude");

    assert_eq!(
        collected(
            &root,
            r#"{ "compilerOptions": { "outDir": "dist" }, "exclude": ["src"] }"#
        ),
        vec!["dist/built.ts", "types/built.ts"]
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn an_out_dir_inherited_through_extends_is_still_excluded() {
    // `outDir` is an ordinary compiler option, so the extending config inherits
    // it and with it the default exclusion: tsgo lists src/index.ts and
    // types/built.ts.
    let root = workspace("extends-out-dir");
    write(
        &root,
        "base.json",
        r#"{ "compilerOptions": { "outDir": "dist" } }"#,
    );

    assert_eq!(
        collected(&root, r#"{ "extends": "./base.json" }"#),
        vec!["src/index.ts", "types/built.ts"]
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn an_absolute_out_dir_is_excluded() {
    // tsgo resolves `outDir` before excluding, so an absolute spelling behaves
    // exactly like the relative one: src/index.ts, types/built.ts.
    let root = workspace("absolute-out-dir");
    let out_dir = root.join("dist").to_string_lossy().replace('\\', "/");

    assert_eq!(
        collected(
            &root,
            &cstr!("{{ \"compilerOptions\": {{ \"outDir\": \"{out_dir}\" }} }}"),
        ),
        vec!["src/index.ts", "types/built.ts"]
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn out_dir_beats_an_include_that_names_it() {
    // `exclude` wins over `include` in tsc, and the default is no different:
    // tsgo lists only src/index.ts.
    let root = workspace("include-out-dir");

    assert_eq!(
        collected(
            &root,
            r#"{ "compilerOptions": { "outDir": "dist" }, "include": ["dist/**/*.ts", "src/**/*.ts"] }"#
        ),
        vec!["src/index.ts"]
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_files_entry_inside_out_dir_bypasses_the_default_exclude() {
    // `files` is not subject to `exclude` at all: tsgo lists dist/built.ts and
    // expands no wildcards.
    let root = workspace("files-in-out-dir");

    assert_eq!(
        collected(
            &root,
            r#"{ "compilerOptions": { "outDir": "dist" }, "files": ["dist/built.ts"] }"#
        ),
        vec!["dist/built.ts"]
    );

    let _ = std::fs::remove_dir_all(&root);
}
