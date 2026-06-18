//! Tests for tsconfig-driven default input collection.

#![allow(clippy::disallowed_macros, clippy::disallowed_types)]

use super::{TsconfigInputCache, load_tsconfig_declaration_options, resolve_extended_tsconfig};
use std::fs;
use std::path::{Path, PathBuf};
use vize_carton::{cstr, path::canonicalize_non_verbatim};

// Each call uses a fresh run-scoped cache, mirroring how an actual `vize
// check` run constructs one `TsconfigInputCache` per invocation.
fn collect_default_check_files(project_root: &Path, tsconfig_path: Option<&Path>) -> Vec<PathBuf> {
    collect_default_check_files_with_jsx(project_root, tsconfig_path, false)
}

fn collect_default_check_files_with_jsx(
    project_root: &Path,
    tsconfig_path: Option<&Path>,
    include_jsx: bool,
) -> Vec<PathBuf> {
    super::collect_default_check_files(
        project_root,
        tsconfig_path,
        include_jsx,
        &mut TsconfigInputCache::default(),
    )
}

fn collect_ambient_declaration_files(
    project_root: &Path,
    tsconfig_path: Option<&Path>,
) -> Vec<PathBuf> {
    super::collect_ambient_declaration_files(
        project_root,
        tsconfig_path,
        &mut TsconfigInputCache::default(),
    )
}

fn resolve_tsconfig_for_files(tsconfig_path: Option<&Path>, files: &[PathBuf]) -> Option<PathBuf> {
    super::resolve_tsconfig_for_files(
        tsconfig_path,
        files,
        false,
        &mut TsconfigInputCache::default(),
    )
}

fn unique_case_dir(name: &str) -> PathBuf {
    static NEXT_CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("vize-tests")
        .join(cstr!("{name}-{}-{case_id}", std::process::id()).as_str())
}

fn relative_paths(root: &Path, files: &[PathBuf]) -> Vec<String> {
    files
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
fn default_collection_respects_include_and_exclude() {
    let case_dir = unique_case_dir("tsconfig-default");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src/generated")).unwrap();
    fs::write(case_dir.join("src/App.vue"), "<template />").unwrap();
    fs::write(case_dir.join("src/main.ts"), "export const ok = true").unwrap();
    fs::write(
        case_dir.join("src/generated/skip.ts"),
        "export const skip = true",
    )
    .unwrap();
    fs::write(case_dir.join("vite.config.ts"), "export default {}").unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{
  "include": ["src/**/*.ts", "src/**/*.vue"],
  "exclude": ["src/generated"]
}"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(
        relative_paths(&case_dir, &files),
        vec!["src/App.vue", "src/main.ts"]
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn default_collection_inherits_extended_include() {
    let case_dir = unique_case_dir("tsconfig-extends");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::write(case_dir.join("src/App.vue"), "<template />").unwrap();
    fs::write(case_dir.join("vite.config.ts"), "export default {}").unwrap();
    fs::write(
        case_dir.join("tsconfig.base.json"),
        r#"{
  "include": ["src/**/*.vue"]
}"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{
  "extends": "./tsconfig.base.json"
}"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(files, vec![case_dir.join("src/App.vue")]);

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn default_collection_matches_parent_relative_extended_include() {
    let case_dir = unique_case_dir("tsconfig-extends-parent-relative");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join(".nuxt")).unwrap();
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::create_dir_all(case_dir.join("dist")).unwrap();
    fs::write(case_dir.join("src/App.vue"), "<template />").unwrap();
    fs::write(case_dir.join("src/main.ts"), "export const ok = true").unwrap();
    fs::write(
        case_dir.join("dist/generated.ts"),
        "export const skip = true",
    )
    .unwrap();
    fs::write(case_dir.join(".nuxt/nuxt.d.ts"), "declare const nuxt: true").unwrap();
    fs::write(
        case_dir.join(".nuxt/tsconfig.json"),
        r#"{
  "include": ["./nuxt.d.ts", "../src/**/*", "../dist/**/*.ts"],
  "exclude": ["../dist"]
}"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{
  "extends": "./.nuxt/tsconfig.json"
}"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(
        relative_paths(&case_dir, &files),
        vec!["src/App.vue", "src/main.ts"]
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn declaration_options_inherit_extends_and_use_config_relative_paths() {
    let case_dir = unique_case_dir("tsconfig-declaration-options");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("configs")).unwrap();
    fs::write(
        case_dir.join("configs/base.json"),
        r#"{
  "compilerOptions": {
"declarationDir": "base-types",
"outDir": "base-dist",
"declarationMap": true
  }
}"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{
  "extends": "./configs/base.json",
  "compilerOptions": {
"outDir": "dist",
"declarationMap": false
  }
}"#,
    )
    .unwrap();

    let options = load_tsconfig_declaration_options(&case_dir.join("tsconfig.json"));

    assert_eq!(
        options.declaration_dir,
        Some(case_dir.join("configs/base-types"))
    );
    assert_eq!(options.out_dir, Some(case_dir.join("dist")));
    assert_eq!(options.declaration_map, Some(false));
    assert_eq!(
        options.output_dir(),
        Some(case_dir.join("configs/base-types").as_path())
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn default_collection_applies_extends_array_in_order() {
    let case_dir = unique_case_dir("tsconfig-extends-array");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src/one")).unwrap();
    fs::create_dir_all(case_dir.join("src/two")).unwrap();
    fs::write(case_dir.join("src/one/One.vue"), "<template />").unwrap();
    fs::write(case_dir.join("src/two/App.vue"), "<template />").unwrap();
    fs::write(case_dir.join("src/two/Skip.vue"), "<template />").unwrap();
    fs::write(
        case_dir.join("tsconfig.one.json"),
        r#"{
  "include": ["src/one/**/*.vue"],
  "exclude": ["src/two/Skip.vue"]
}"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("tsconfig.two.json"),
        r#"{
  "include": ["src/two/**/*.vue"]
}"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{
  "extends": ["./tsconfig.one.json", "./tsconfig.two.json"]
}"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(files, vec![case_dir.join("src/two/App.vue")]);

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn extended_config_resolution_finds_ancestor_node_modules() {
    let case_dir = unique_case_dir("tsconfig-package-extends");
    let _ = fs::remove_dir_all(&case_dir);
    let app_dir = case_dir.join("packages/app");
    let package_dir = case_dir.join("node_modules/@scope/tsconfig");
    fs::create_dir_all(&app_dir).unwrap();
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(app_dir.join("tsconfig.json"), "{}").unwrap();
    fs::write(
        package_dir.join("tsconfig.vue.json"),
        r#"{
  "compilerOptions": {
"strict": true
  }
}"#,
    )
    .unwrap();

    let resolved = resolve_extended_tsconfig(
        &app_dir.join("tsconfig.json"),
        "@scope/tsconfig/tsconfig.vue.json",
    );

    assert_eq!(resolved, Some(package_dir.join("tsconfig.vue.json")));

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn extended_config_resolution_uses_package_json_tsconfig_field() {
    let case_dir = unique_case_dir("tsconfig-package-json-field");
    let _ = fs::remove_dir_all(&case_dir);
    let app_dir = case_dir.join("packages/app");
    let package_dir = case_dir.join("node_modules/@scope/tsconfig");
    fs::create_dir_all(app_dir.join("src")).unwrap();
    fs::create_dir_all(package_dir.join("configs")).unwrap();
    fs::write(app_dir.join("tsconfig.json"), "{}").unwrap();
    fs::write(
        package_dir.join("package.json"),
        r#"{
  "name": "@scope/tsconfig",
  "tsconfig": "configs/vue.json"
}"#,
    )
    .unwrap();
    fs::write(
        package_dir.join("configs/vue.json"),
        r#"{
  "compilerOptions": {
"strict": true
  }
}"#,
    )
    .unwrap();
    fs::write(package_dir.join("tsconfig.json"), "{}").unwrap();

    let resolved = resolve_extended_tsconfig(&app_dir.join("tsconfig.json"), "@scope/tsconfig");

    assert_eq!(resolved, Some(package_dir.join("configs/vue.json")));

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn default_collection_skips_nuxt_import_manifest_files_entries() {
    let case_dir = unique_case_dir("tsconfig-nuxt-import-manifest-files");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join(".nuxt/types")).unwrap();
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::write(case_dir.join("src/App.vue"), "<template />").unwrap();
    fs::write(
        case_dir.join(".nuxt/imports.d.ts"),
        "export { useVfjsI18n } from '../composables/useVfjsI18n'\n",
    )
    .unwrap();
    fs::write(
        case_dir.join(".nuxt/types/imports.d.ts"),
        "declare global { const useVfjsI18n: typeof import('../composables/useVfjsI18n')['useVfjsI18n'] }\nexport {}\n",
    )
    .unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{
  "files": [
    "src/App.vue",
    ".nuxt/imports.d.ts",
    ".nuxt/types/imports.d.ts"
  ]
}"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(relative_paths(&case_dir, &files), vec!["src/App.vue"]);

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn ambient_declaration_collection_keeps_only_dts_within_include() {
    let case_dir = unique_case_dir("tsconfig-ambient-dts");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src/@types")).unwrap();
    fs::write(
        case_dir.join("src/@types/globals.d.ts"),
        "export {};\ndeclare global { type GlobalTabType = 'a' | 'b'; }\n",
    )
    .unwrap();
    fs::write(case_dir.join("src/env.d.ts"), "declare const X: string;").unwrap();
    fs::write(case_dir.join("src/App.vue"), "<template />").unwrap();
    fs::write(case_dir.join("src/main.ts"), "export const ok = true").unwrap();
    fs::write(case_dir.join("outside.d.ts"), "declare const Y: string;").unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();

    let files = collect_ambient_declaration_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(
        relative_paths(&case_dir, &files),
        vec!["src/@types/globals.d.ts", "src/env.d.ts"]
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn ambient_declaration_collection_keeps_project_shims_but_skips_vue_shadows() {
    let case_dir = unique_case_dir("tsconfig-module-shim-dts");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::create_dir_all(case_dir.join(".nuxt/types")).unwrap();
    // This file would shadow the real `vue` package if force-loaded as a
    // program root, so it must remain excluded.
    fs::write(
        case_dir.join("src/vue-shadow.d.ts"),
        "declare module \"vue\" {\n  export interface GlobalComponents {}\n}\n",
    )
    .unwrap();
    // Project shims are needed for explicit checks: no source import can
    // discover these declarations otherwise.
    fs::write(
        case_dir.join("src/project-shims.d.ts"),
        "declare module \"*.css\";\ndeclare module \"~icons/foo\";\n",
    )
    .unwrap();
    // Nuxt/Vue package augmentations are safe when the declaration file is
    // an external module.
    fs::write(
        case_dir.join("src/vue-augmentation.d.ts"),
        "import \"vue\";\ndeclare module \"vue\" {\n  export interface GlobalComponents {}\n}\nexport {};\n",
    )
    .unwrap();
    // Genuine ambient-global file: must still be collected.
    fs::write(
        case_dir.join("src/globals.d.ts"),
        "export {};\ndeclare global { type GlobalTabType = 'a' | 'b'; }\n",
    )
    .unwrap();
    // Namespace-style `declare module Foo` is a plain global, not a shim.
    fs::write(
        case_dir.join("src/namespace.d.ts"),
        "declare module Foo { const bar: string; }\n",
    )
    .unwrap();
    // Hidden tsconfig roots such as `.nuxt` are excluded by the normal
    // default scanner but must still be loaded as ambient roots.
    fs::write(
        case_dir.join(".nuxt/nuxt.d.ts"),
        "/// <reference path=\"types/feature-flags.d.ts\" />\nexport {};\n",
    )
    .unwrap();
    fs::write(
        case_dir.join(".nuxt/types/feature-flags.d.ts"),
        "export {};\ndeclare global { interface ImportMeta { vfFeatures: { enabled: boolean }; } }\n",
    )
    .unwrap();
    fs::write(case_dir.join("src/App.vue"), "<template />").unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{
  "include": ["src/**/*", ".nuxt/nuxt.d.ts"]
}"#,
    )
    .unwrap();

    let files = collect_ambient_declaration_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(
        relative_paths(&case_dir, &files),
        vec![
            "src/globals.d.ts",
            "src/namespace.d.ts",
            "src/project-shims.d.ts",
            "src/vue-augmentation.d.ts",
            ".nuxt/types/feature-flags.d.ts",
        ]
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn default_collection_uses_files_entries() {
    let case_dir = unique_case_dir("tsconfig-files");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::write(case_dir.join("src/entry.ts"), "export const ok = true").unwrap();
    fs::write(case_dir.join("src/extra.ts"), "export const extra = true").unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{
  "files": ["src/entry.ts"]
}"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(files, vec![case_dir.join("src/entry.ts")]);

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn default_collection_follows_referenced_tsconfigs() {
    let case_dir = unique_case_dir("tsconfig-references");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join(".generated")).unwrap();
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::write(case_dir.join("src/App.vue"), "<template />").unwrap();
    fs::write(
        case_dir.join(".generated/types.d.ts"),
        "declare const X: true",
    )
    .unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{
  "files": [],
  "references": [{ "path": "./.generated/tsconfig.app.json" }]
}"#,
    )
    .unwrap();
    fs::write(
        case_dir.join(".generated/tsconfig.app.json"),
        r#"{
  "include": ["./types.d.ts", "../src/**/*.vue"]
}"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(relative_paths(&case_dir, &files), vec!["src/App.vue"]);

    let ambient =
        collect_ambient_declaration_files(&case_dir, Some(&case_dir.join("tsconfig.json")));
    assert_eq!(
        relative_paths(&case_dir, &ambient),
        vec![".generated/types.d.ts"]
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn tsconfig_for_files_uses_referenced_owner() {
    let case_dir = unique_case_dir("tsconfig-reference-owner");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join(".generated")).unwrap();
    fs::create_dir_all(case_dir.join("src")).unwrap();
    let app = case_dir.join("src/App.vue");
    fs::write(&app, "<template />").unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{
  "files": [],
  "references": [{ "path": "./.generated/tsconfig.app.json" }]
}"#,
    )
    .unwrap();
    fs::write(
        case_dir.join(".generated/tsconfig.app.json"),
        r#"{
  "include": ["../src/**/*.vue"]
}"#,
    )
    .unwrap();

    let owner = resolve_tsconfig_for_files(Some(&case_dir.join("tsconfig.json")), &[app]);

    assert_eq!(owner, Some(case_dir.join(".generated/tsconfig.app.json")));

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn jsonc_comments_and_trailing_commas_are_stripped_before_parsing() {
    // If the JSONC stripping failed, the tsconfig would parse as null, fall
    // back to an implicit `**/*` scan, and `src/skip.ts` would be collected.
    // Asserting it is excluded proves the comment/trailing-comma stripping ran.
    let case_dir = unique_case_dir("tsconfig-jsonc-comments");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::write(case_dir.join("src/keep.ts"), "export const keep = true").unwrap();
    fs::write(case_dir.join("src/skip.ts"), "export const skip = true").unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        "{\n  // leading line comment\n  \"include\": [\"src/**/*.ts\"], /* trailing block\n  comment spanning lines */\n  \"exclude\": [\n    \"src/skip.ts\",\n  ],\n}\n",
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(relative_paths(&case_dir, &files), vec!["src/keep.ts"]);

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn jsonc_does_not_strip_comment_like_sequences_inside_strings() {
    // The exclude value literally contains `//`; if the comment stripper
    // ignored string boundaries it would truncate the pattern and stop
    // excluding the file.
    let case_dir = unique_case_dir("tsconfig-jsonc-string");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::write(case_dir.join("src/a.ts"), "export const a = true").unwrap();
    fs::write(case_dir.join("src/skip.ts"), "export const skip = true").unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{
  "include": ["src/**/*.ts"],
  "exclude": ["src/skip.ts"]
}"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(relative_paths(&case_dir, &files), vec!["src/a.ts"]);

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn single_star_include_does_not_cross_directory_separator() {
    let case_dir = unique_case_dir("tsconfig-glob-single-star");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src/nested")).unwrap();
    fs::write(case_dir.join("src/top.ts"), "export const top = true").unwrap();
    fs::write(
        case_dir.join("src/nested/deep.ts"),
        "export const deep = true",
    )
    .unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{ "include": ["src/*.ts"] }"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(relative_paths(&case_dir, &files), vec!["src/top.ts"]);

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn bare_directory_include_expands_to_recursive_glob() {
    let case_dir = unique_case_dir("tsconfig-glob-bare-dir");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src/nested")).unwrap();
    fs::write(case_dir.join("src/top.ts"), "export const top = true").unwrap();
    fs::write(
        case_dir.join("src/nested/deep.ts"),
        "export const deep = true",
    )
    .unwrap();
    fs::write(case_dir.join("tsconfig.json"), r#"{ "include": ["src"] }"#).unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(
        relative_paths(&case_dir, &files),
        vec!["src/nested/deep.ts", "src/top.ts"]
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn dot_include_matches_every_supported_file() {
    let case_dir = unique_case_dir("tsconfig-glob-dot");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("nested")).unwrap();
    fs::write(case_dir.join("root.ts"), "export const root = true").unwrap();
    fs::write(case_dir.join("nested/leaf.ts"), "export const leaf = true").unwrap();
    fs::write(case_dir.join("tsconfig.json"), r#"{ "include": ["."] }"#).unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(
        relative_paths(&case_dir, &files),
        vec!["nested/leaf.ts", "root.ts"]
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn leading_dot_slash_include_is_normalized() {
    let case_dir = unique_case_dir("tsconfig-glob-dot-slash");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::write(case_dir.join("src/a.ts"), "export const a = true").unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{ "include": ["./src/**/*.ts"] }"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(relative_paths(&case_dir, &files), vec!["src/a.ts"]);

    let _ = fs::remove_dir_all(&case_dir);
}

#[cfg(not(windows))]
#[test]
fn include_glob_matching_is_case_sensitive_on_unix() {
    let case_dir = unique_case_dir("tsconfig-glob-case");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::write(case_dir.join("src/a.ts"), "export const a = true").unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{ "include": ["SRC/**/*.ts"] }"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert!(
        files.is_empty(),
        "an upper-case include must not match a lower-case directory on Unix: {files:?}"
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn declaration_files_are_always_supported() {
    let case_dir = unique_case_dir("tsconfig-ext-dts");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::write(case_dir.join("src/env.d.ts"), "declare const X: string;").unwrap();
    fs::write(case_dir.join("src/a.ts"), "export const a = true").unwrap();
    fs::write(case_dir.join("src/ignore.js"), "module.exports = {}").unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{ "include": ["src/**/*"] }"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(
        relative_paths(&case_dir, &files),
        vec!["src/a.ts", "src/env.d.ts"]
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn supported_extensions_cover_ts_family_and_reject_js_family() {
    let case_dir = unique_case_dir("tsconfig-ext-family");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    let supported = ["App.vue", "a.ts", "b.tsx", "c.mts", "d.cts"];
    let unsupported = ["e.js", "f.jsx", "g.cjs", "h.mjs", "data.json"];
    for name in supported.iter().chain(unsupported.iter()) {
        fs::write(case_dir.join("src").join(name), "x").unwrap();
    }
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{ "include": ["src/**/*"] }"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(
        files,
        vec![
            case_dir.join("src/App.vue"),
            case_dir.join("src/a.ts"),
            case_dir.join("src/b.tsx"),
            case_dir.join("src/c.mts"),
            case_dir.join("src/d.cts"),
        ]
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn jsx_extension_is_collected_only_for_jsx_typecheck() {
    let case_dir = unique_case_dir("tsconfig-ext-jsx");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::write(case_dir.join("src/App.jsx"), "const App = () => <div />").unwrap();
    fs::write(case_dir.join("src/App.tsx"), "const App = () => <div />").unwrap();
    fs::write(case_dir.join("src/skip.js"), "export const skip = true").unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{ "include": ["src/**/*"] }"#,
    )
    .unwrap();

    let without_jsx = collect_default_check_files_with_jsx(
        &case_dir,
        Some(&case_dir.join("tsconfig.json")),
        false,
    );
    let with_jsx = collect_default_check_files_with_jsx(
        &case_dir,
        Some(&case_dir.join("tsconfig.json")),
        true,
    );

    assert_eq!(without_jsx, vec![case_dir.join("src/App.tsx")]);
    assert_eq!(
        with_jsx,
        vec![case_dir.join("src/App.jsx"), case_dir.join("src/App.tsx")]
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn malformed_tsconfig_falls_back_to_full_default_scan() {
    // Unparseable JSON degrades to an implicit `**/*` include with the
    // default excludes (node_modules), so source is still collected while
    // dependencies are not.
    let case_dir = unique_case_dir("tsconfig-malformed");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::create_dir_all(case_dir.join("node_modules/dep")).unwrap();
    fs::write(case_dir.join("src/a.ts"), "export const a = true").unwrap();
    fs::write(
        case_dir.join("node_modules/dep/index.ts"),
        "export const dep = true",
    )
    .unwrap();
    fs::write(case_dir.join("tsconfig.json"), "{ this is not valid json").unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(relative_paths(&case_dir, &files), vec!["src/a.ts"]);

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn custom_exclude_glob_drops_a_matching_subtree() {
    // An explicit exclude is honored for a non-ignored subtree, independent
    // of the default node_modules/bower_components excludes.
    let case_dir = unique_case_dir("tsconfig-custom-exclude");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src/keep")).unwrap();
    fs::create_dir_all(case_dir.join("src/skip")).unwrap();
    fs::write(case_dir.join("src/keep/a.ts"), "export const a = true").unwrap();
    fs::write(case_dir.join("src/skip/b.ts"), "export const b = true").unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{ "include": ["src/**/*.ts"], "exclude": ["src/skip"] }"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(relative_paths(&case_dir, &files), vec!["src/keep/a.ts"]);

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn exclude_takes_precedence_over_include_for_the_same_file() {
    let case_dir = unique_case_dir("tsconfig-exclude-precedence");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::write(case_dir.join("src/a.ts"), "export const a = true").unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{ "include": ["src/**/*.ts"], "exclude": ["src/**/*.ts"] }"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert!(
        files.is_empty(),
        "exclude should win over include for the same file: {files:?}"
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn files_entries_bypass_exclude_filtering() {
    let case_dir = unique_case_dir("tsconfig-files-bypass-exclude");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::write(case_dir.join("src/a.ts"), "export const a = true").unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{ "files": ["src/a.ts"], "exclude": ["src/**/*"] }"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(files, vec![case_dir.join("src/a.ts")]);

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn files_present_suppresses_the_implicit_wildcard_scan() {
    let case_dir = unique_case_dir("tsconfig-files-suppress-scan");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::write(case_dir.join("src/a.ts"), "export const a = true").unwrap();
    fs::write(case_dir.join("src/b.ts"), "export const b = true").unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{ "files": ["src/a.ts"] }"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(files, vec![case_dir.join("src/a.ts")]);

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn files_entry_with_unsupported_extension_is_dropped() {
    let case_dir = unique_case_dir("tsconfig-files-bad-ext");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::write(case_dir.join("src/x.js"), "module.exports = {}").unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{ "files": ["src/x.js"] }"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert!(
        files.is_empty(),
        "unsupported files entry should drop: {files:?}"
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn parent_relative_tsconfig_entries_are_collected() {
    let case_dir = unique_case_dir("tsconfig-parent-relative-inputs");
    let _ = fs::remove_dir_all(&case_dir);
    let generated_dir = case_dir.join(".generated");
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::create_dir_all(&generated_dir).unwrap();
    fs::write(case_dir.join("src/bad.ts"), "export const bad = true").unwrap();
    for (name, json) in [
        ("files.json", r#"{ "files": ["../src/bad.ts"] }"#),
        ("include.json", r#"{ "include": ["../src/**/*.ts"] }"#),
    ] {
        fs::write(generated_dir.join(name), json).unwrap();
        let files = collect_default_check_files(&generated_dir, Some(&generated_dir.join(name)));
        assert_eq!(relative_paths(&case_dir, &files), vec!["src/bad.ts"]);
    }

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn circular_extends_chain_terminates_and_applies_host_include() {
    let case_dir = unique_case_dir("tsconfig-circular-extends");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::write(case_dir.join("src/a.ts"), "export const a = true").unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{ "extends": "./other.json", "include": ["src/**/*.ts"] }"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("other.json"),
        r#"{ "extends": "./tsconfig.json" }"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(relative_paths(&case_dir, &files), vec!["src/a.ts"]);

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn missing_extends_target_is_skipped() {
    let case_dir = unique_case_dir("tsconfig-missing-extends");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::write(case_dir.join("src/a.ts"), "export const a = true").unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{ "extends": "./does-not-exist.json", "include": ["src/**/*.ts"] }"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(relative_paths(&case_dir, &files), vec!["src/a.ts"]);

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn circular_references_chain_terminates_and_each_project_contributes() {
    let case_dir = unique_case_dir("tsconfig-circular-references");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("a")).unwrap();
    fs::create_dir_all(case_dir.join("b")).unwrap();
    fs::write(case_dir.join("a/x.ts"), "export const x = true").unwrap();
    fs::write(case_dir.join("b/y.ts"), "export const y = true").unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{ "files": [], "include": ["a/**/*.ts"], "references": [{ "path": "./b.json" }] }"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("b.json"),
        r#"{ "include": ["b/**/*.ts"], "references": [{ "path": "./tsconfig.json" }] }"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(relative_paths(&case_dir, &files), vec!["a/x.ts", "b/y.ts"]);

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn reference_path_to_directory_resolves_to_tsconfig_json() {
    let case_dir = unique_case_dir("tsconfig-reference-dir");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("sub")).unwrap();
    fs::write(case_dir.join("sub/z.ts"), "export const z = true").unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{ "files": [], "references": [{ "path": "./sub" }] }"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("sub/tsconfig.json"),
        r#"{ "include": ["*.ts"] }"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(relative_paths(&case_dir, &files), vec!["sub/z.ts"]);

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn owner_resolution_returns_root_for_no_supported_files() {
    let case_dir = unique_case_dir("tsconfig-owner-no-files");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(&case_dir).unwrap();
    let root = case_dir.join("tsconfig.json");
    fs::write(&root, r#"{ "include": ["src/**/*.ts"] }"#).unwrap();

    let normalized_root = canonicalize_non_verbatim(&root);

    // Empty file list -> root.
    assert_eq!(
        resolve_tsconfig_for_files(Some(&root), &[]),
        Some(normalized_root.clone())
    );

    // Unsupported-only file list -> root (the .js is filtered out first).
    assert_eq!(
        resolve_tsconfig_for_files(Some(&root), &[case_dir.join("src/app.js")]),
        Some(normalized_root)
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn owner_resolution_falls_back_to_root_when_files_span_projects() {
    let case_dir = unique_case_dir("tsconfig-owner-split");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("a")).unwrap();
    fs::create_dir_all(case_dir.join("b")).unwrap();
    let a_file = case_dir.join("a/x.ts");
    let b_file = case_dir.join("b/y.ts");
    fs::write(&a_file, "export const x = true").unwrap();
    fs::write(&b_file, "export const y = true").unwrap();
    let root = case_dir.join("tsconfig.json");
    fs::write(
        &root,
        r#"{ "include": ["root-only/**/*.ts"], "references": [{ "path": "./a.json" }, { "path": "./b.json" }] }"#,
    )
    .unwrap();
    fs::write(case_dir.join("a.json"), r#"{ "include": ["a/**/*.ts"] }"#).unwrap();
    fs::write(case_dir.join("b.json"), r#"{ "include": ["b/**/*.ts"] }"#).unwrap();

    let owner = resolve_tsconfig_for_files(Some(&root), &[a_file, b_file]);

    assert_eq!(owner, Some(canonicalize_non_verbatim(&root)));

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn owner_resolution_falls_back_to_root_for_an_unowned_file() {
    let case_dir = unique_case_dir("tsconfig-owner-unowned");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("c")).unwrap();
    let unowned = case_dir.join("c/z.ts");
    fs::write(&unowned, "export const z = true").unwrap();
    let root = case_dir.join("tsconfig.json");
    fs::write(
        &root,
        r#"{ "include": ["root-only/**/*.ts"], "references": [{ "path": "./a.json" }] }"#,
    )
    .unwrap();
    fs::write(case_dir.join("a.json"), r#"{ "include": ["a/**/*.ts"] }"#).unwrap();

    let owner = resolve_tsconfig_for_files(Some(&root), &[unowned]);

    assert_eq!(owner, Some(canonicalize_non_verbatim(&root)));

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn shared_run_cache_matches_fresh_cache_results() {
    let case_dir = unique_case_dir("tsconfig-shared-cache");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    let app = case_dir.join("src/App.vue");
    let main = case_dir.join("src/main.ts");
    fs::write(&app, "<template />").unwrap();
    fs::write(&main, "export const ok = true").unwrap();
    fs::write(
        case_dir.join("src/globals.d.ts"),
        "declare const FLAG: boolean",
    )
    .unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{ "include": ["src/**/*.ts", "src/**/*.vue", "src/**/*.d.ts"] }"#,
    )
    .unwrap();
    let tsconfig = case_dir.join("tsconfig.json");

    // One run-scoped cache shared across owner resolution (twice), default
    // collection, and ambient collection must match per-call fresh caches.
    let mut cache = TsconfigInputCache::default();
    let owner_shared = super::resolve_tsconfig_for_files(
        Some(&tsconfig),
        &[app.clone(), main.clone()],
        false,
        &mut cache,
    );
    let owner_shared_again =
        super::resolve_tsconfig_for_files(Some(&tsconfig), &[app.clone()], false, &mut cache);
    let files_shared =
        super::collect_default_check_files(&case_dir, Some(&tsconfig), false, &mut cache);
    let ambient_shared =
        super::collect_ambient_declaration_files(&case_dir, Some(&tsconfig), &mut cache);

    assert_eq!(
        owner_shared,
        resolve_tsconfig_for_files(Some(&tsconfig), &[app.clone(), main])
    );
    assert_eq!(
        owner_shared_again,
        resolve_tsconfig_for_files(Some(&tsconfig), &[app])
    );
    assert_eq!(
        files_shared,
        collect_default_check_files(&case_dir, Some(&tsconfig))
    );
    assert_eq!(
        ambient_shared,
        collect_ambient_declaration_files(&case_dir, Some(&tsconfig))
    );
    assert_eq!(files_shared.len(), 3);
    assert_eq!(ambient_shared.len(), 1);

    let _ = fs::remove_dir_all(&case_dir);
}
