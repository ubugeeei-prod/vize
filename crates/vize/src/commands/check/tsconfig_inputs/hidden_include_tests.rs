//! Hidden include-root tests split out from the large tsconfig suite.

#![allow(clippy::disallowed_macros, clippy::disallowed_types)]

use std::{
    fs,
    path::{Path, PathBuf},
};

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
fn default_collection_checks_explicitly_included_hidden_directories() {
    // `tsc` drops dot-directories only while expanding wildcards. A literal
    // `.vitepress` segment in an include pattern is matched literally, so
    // VitePress/Storybook sources are part of the program. Skipping them made
    // `vize check` silently report nothing for a whole docs tree while
    // `vue-tsc` reported real errors there.
    let case_dir = unique_case_dir("tsconfig-hidden-include-root");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("docs/.vitepress/theme/components")).unwrap();
    fs::create_dir_all(case_dir.join("docs/.vitepress/nested/deep")).unwrap();
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::write(
        case_dir.join("docs/.vitepress/theme/components/Logo.vue"),
        "<template />",
    )
    .unwrap();
    fs::write(
        case_dir.join("docs/.vitepress/theme/index.ts"),
        "export const theme = true",
    )
    .unwrap();
    // Outside the included subtree: must stay out even though it is under the
    // same hidden directory.
    fs::write(
        case_dir.join("docs/.vitepress/nested/deep/other.ts"),
        "export const other = true",
    )
    .unwrap();
    fs::write(case_dir.join("src/App.vue"), "<template />").unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{
  "include": ["docs/.vitepress/theme", "src"]
}"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, &case_dir.join("tsconfig.json"));

    assert_eq!(
        relative_paths(&case_dir, &files),
        vec![
            "docs/.vitepress/theme/components/Logo.vue",
            "docs/.vitepress/theme/index.ts",
            "src/App.vue",
        ]
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn default_collection_still_skips_hidden_directories_matched_only_by_a_wildcard() {
    // The counterpart to the case above: when no include segment names the
    // hidden directory, `tsc`'s wildcard expansion skips it, and so must ours.
    let case_dir = unique_case_dir("tsconfig-hidden-wildcard-only");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("docs/.vitepress/theme")).unwrap();
    fs::create_dir_all(case_dir.join("docs/public")).unwrap();
    fs::write(
        case_dir.join("docs/.vitepress/theme/index.ts"),
        "export const theme = true",
    )
    .unwrap();
    fs::write(
        case_dir.join("docs/public/visible.ts"),
        "export const visible = true",
    )
    .unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{
  "include": ["docs/**/*"]
}"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, &case_dir.join("tsconfig.json"));

    assert_eq!(
        relative_paths(&case_dir, &files),
        vec!["docs/public/visible.ts"]
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn default_collection_keeps_hidden_root_declarations_out_of_the_checked_set() {
    // Nuxt's root tsconfig names its generated declarations literally
    // (`.nuxt/components.d.cts`). Those are ambient program inputs that
    // `collect_hidden_ambient_declaration_files` loads, so listing them as
    // checked sources would report a file vue-tsc never diagnoses.
    let case_dir = unique_case_dir("tsconfig-hidden-root-declarations");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join(".nuxt")).unwrap();
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::write(
        case_dir.join(".nuxt/components.d.cts"),
        "declare module 'vue' {}\nexport {}\n",
    )
    .unwrap();
    fs::write(
        case_dir.join(".nuxt/plugins.ts"),
        "export const plugins = []",
    )
    .unwrap();
    fs::write(case_dir.join("src/App.vue"), "<template />").unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{
  "include": [".nuxt/components.d.cts", ".nuxt/plugins.ts", "src"]
}"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, &case_dir.join("tsconfig.json"));

    assert_eq!(
        relative_paths(&case_dir, &files),
        vec![".nuxt/plugins.ts", "src/App.vue"]
    );

    let _ = fs::remove_dir_all(&case_dir);
}
