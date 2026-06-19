use super::super::ignores::CheckIgnoreSet;
use super::{
    base_dir_from_pattern, collect_check_files, collect_check_files_with_ignores, collect_vue_files,
};
use crate::commands::check::{
    imports::collect_transitive_local_imports, imports_aliases::PathAliasResolver,
    path_cache::CanonicalPathCache,
};
use std::fs;
use std::path::{Path, PathBuf};
use vize_carton::cstr;

fn unique_case_dir(name: &str) -> PathBuf {
    static NEXT_CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("vize-tests")
        .join(cstr!("{name}-{}-{case_id}", std::process::id()).as_str())
}

fn write_file(root: &Path, rel: &str, contents: &str) -> PathBuf {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, contents).unwrap();
    path
}

#[test]
fn base_dir_from_glob_patterns() {
    assert_eq!(
        base_dir_from_pattern("./src/**/*.vue"),
        PathBuf::from("./src")
    );
    assert_eq!(base_dir_from_pattern("."), PathBuf::from("."));
}

#[test]
fn collect_check_files_includes_ts_and_vue_and_dts() {
    let case_dir = unique_case_dir("collect-check");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::write(case_dir.join("src/App.vue"), "").unwrap();
    fs::write(case_dir.join("src/Component.jsx"), "").unwrap();
    fs::write(case_dir.join("src/main.ts"), "").unwrap();
    fs::write(case_dir.join("src/env.d.ts"), "").unwrap();
    fs::write(case_dir.join("src/skip.js"), "").unwrap();

    let files = collect_check_files(&vec![case_dir.display().to_string()], false);

    assert_eq!(
        files,
        vec![
            case_dir.join("src/App.vue"),
            case_dir.join("src/env.d.ts"),
            case_dir.join("src/main.ts"),
        ]
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn collect_check_files_includes_jsx_only_when_enabled() {
    let case_dir = unique_case_dir("collect-check-jsx");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::write(case_dir.join("src/App.jsx"), "").unwrap();
    fs::write(case_dir.join("src/App.tsx"), "").unwrap();
    fs::write(case_dir.join("src/skip.js"), "").unwrap();

    let files = collect_check_files(&vec![case_dir.display().to_string()], true);

    assert_eq!(
        files,
        vec![case_dir.join("src/App.jsx"), case_dir.join("src/App.tsx")]
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn collect_check_files_filters_quoted_globs() {
    let case_dir = unique_case_dir("collect-check-glob");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src/nested")).unwrap();
    fs::write(case_dir.join("src/App.vue"), "").unwrap();
    fs::write(case_dir.join("src/main.ts"), "").unwrap();
    fs::write(case_dir.join("src/nested/View.vue"), "").unwrap();
    fs::write(case_dir.join("src/nested/model.ts"), "").unwrap();

    let files = collect_check_files(
        &vec![case_dir.join("src/**/*.vue").display().to_string()],
        false,
    );

    assert_eq!(
        files,
        vec![
            case_dir.join("src/App.vue"),
            case_dir.join("src/nested/View.vue"),
        ]
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn collect_check_files_applies_entry_ignores() {
    let case_dir = unique_case_dir("collect-check-entry-ignores");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::create_dir_all(case_dir.join("design-system/src")).unwrap();
    fs::write(case_dir.join("src/App.vue"), "").unwrap();
    fs::write(case_dir.join("src/Ignored.vue"), "").unwrap();
    fs::write(case_dir.join("design-system/src/Kept.vue"), "").unwrap();
    fs::write(case_dir.join("design-system/src/Fixture.vue"), "").unwrap();

    let ignore_set = CheckIgnoreSet::new(
        &[
            crate::config::ConfigEntryIgnore {
                base_path: None,
                pattern: "src/Ignored.vue".into(),
            },
            crate::config::ConfigEntryIgnore {
                base_path: Some("design-system".into()),
                pattern: "src/Fixture.vue".into(),
            },
        ],
        &case_dir,
    );

    let files = collect_check_files_with_ignores(
        &vec![case_dir.display().to_string()],
        false,
        ignore_set.as_ref(),
    );
    let explicit = collect_check_files_with_ignores(
        &vec![case_dir.join("src/Ignored.vue").display().to_string()],
        false,
        ignore_set.as_ref(),
    );

    assert_eq!(
        files,
        vec![
            case_dir.join("design-system/src/Kept.vue"),
            case_dir.join("src/App.vue"),
        ]
    );
    assert!(explicit.is_empty());

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn collect_check_files_normalizes_entry_ignore_paths_and_duplicates() {
    let case_dir = unique_case_dir("collect-check-entry-ignore-paths");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src/nested")).unwrap();
    fs::create_dir_all(case_dir.join("packages/admin/src")).unwrap();
    let app = write_file(&case_dir, "src/App.vue", "");
    let nested = write_file(&case_dir, "src/nested/Panel.vue", "");
    write_file(&case_dir, "src/Ignored.vue", "");
    write_file(&case_dir, "src/nested/Ignored.vue", "");
    write_file(&case_dir, "packages/admin/src/Ignored.vue", "");
    let admin_page = write_file(&case_dir, "packages/admin/src/Page.vue", "");

    let ignore_set = CheckIgnoreSet::new(
        &[
            crate::config::ConfigEntryIgnore {
                base_path: None,
                pattern: case_dir
                    .join("src/Ignored.vue")
                    .to_string_lossy()
                    .into_owned()
                    .into(),
            },
            crate::config::ConfigEntryIgnore {
                base_path: None,
                pattern: "src/nested/Ignored.vue".into(),
            },
            crate::config::ConfigEntryIgnore {
                base_path: Some("packages/admin".into()),
                pattern: "src\\Ignored.vue".into(),
            },
        ],
        &case_dir,
    );

    let files = collect_check_files_with_ignores(
        &vec![
            case_dir.join("src").display().to_string(),
            case_dir.join("src/./App.vue").display().to_string(),
            case_dir.join("packages/admin/src").display().to_string(),
            case_dir
                .join("packages/admin/src/Ignored.vue")
                .display()
                .to_string(),
        ],
        false,
        ignore_set.as_ref(),
    );

    assert_eq!(files, vec![admin_page, app, nested]);

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn path_root_filter_drops_alias_imports_outside_package_cwd() {
    let workspace = unique_case_dir("package-transitive-alias-root");
    let _ = fs::remove_dir_all(&workspace);
    fs::create_dir_all(&workspace).unwrap();
    fs::write(
        workspace.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": { "~/*": ["*"] }
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
    let _root_only = write_file(
        &workspace,
        "src/generated/tecack/custom.ts",
        "export const rootOnly = 'root';\n",
    );

    let package_root = workspace.join("devtools");
    fs::create_dir_all(&package_root).unwrap();
    fs::write(
        package_root.join("tsconfig.json"),
        r#"{
  "extends": "../tsconfig.json",
  "include": ["src/**/*.vue", "src/**/*.ts"]
}"#,
    )
    .unwrap();
    let app = write_file(
        &package_root,
        "src/App.vue",
        r#"<script setup lang="ts">
import { rootOnly } from "~/src/generated/tecack/custom";
void rootOnly;
</script>
"#,
    );

    let aliases = PathAliasResolver::from_tsconfig(Some(&package_root.join("tsconfig.json")));
    let discovered = collect_transitive_local_imports(
        std::slice::from_ref(&app),
        &package_root,
        &mut CanonicalPathCache::default(),
        false,
        Some(&aliases),
    );

    assert!(
        discovered.is_empty(),
        "root app import leaked into package inputs: {discovered:#?}"
    );

    let _ = fs::remove_dir_all(&workspace);
}

#[test]
fn collect_vue_files_stays_vue_only() {
    let case_dir = unique_case_dir("collect-vue");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::write(case_dir.join("src/App.vue"), "").unwrap();
    fs::write(case_dir.join("src/main.ts"), "").unwrap();

    let files = collect_vue_files(&vec![case_dir.display().to_string()]);

    assert_eq!(files, vec![case_dir.join("src/App.vue")]);

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn collect_vue_files_filters_quoted_globs() {
    let case_dir = unique_case_dir("collect-vue-glob");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src/nested")).unwrap();
    fs::write(case_dir.join("src/App.vue"), "").unwrap();
    fs::write(case_dir.join("src/nested/View.vue"), "").unwrap();
    fs::write(case_dir.join("src/nested/Skip.vue"), "").unwrap();

    let files = collect_vue_files(&vec![
        case_dir.join("src/nested/*.vue").display().to_string(),
    ]);

    assert_eq!(
        files,
        vec![
            case_dir.join("src/nested/Skip.vue"),
            case_dir.join("src/nested/View.vue"),
        ]
    );

    let _ = fs::remove_dir_all(&case_dir);
}
