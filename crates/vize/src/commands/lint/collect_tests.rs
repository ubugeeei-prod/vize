use super::{LintIgnoreSet, base_dir_from_lint_pattern, collect_lint_file_collection};
use std::fs;

#[test]
fn root_level_glob_base_dir_is_current_directory() {
    assert_eq!(
        base_dir_from_lint_pattern("foo*.vue"),
        std::path::PathBuf::from(".")
    );
    assert_eq!(
        base_dir_from_lint_pattern("*.vue"),
        std::path::PathBuf::from(".")
    );
    assert_eq!(
        base_dir_from_lint_pattern("src/foo*.vue"),
        std::path::PathBuf::from("src")
    );
}

#[test]
fn windows_absolute_glob_base_dir_uses_parent_directory() {
    assert_eq!(
        base_dir_from_lint_pattern(r"C:\work\src\foo*.vue"),
        std::path::PathBuf::from("C:/work/src")
    );
    assert_eq!(
        base_dir_from_lint_pattern(r"C:\*.vue"),
        std::path::PathBuf::from("C:/")
    );
}

#[test]
fn collection_includes_vue_html_scripts_and_jsx() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("App.vue"), "").unwrap();
    fs::write(src.join("index.html"), "").unwrap();
    fs::write(src.join("config.js"), "").unwrap();
    fs::write(src.join("store.ts"), "").unwrap();
    fs::write(src.join("Panel.jsx"), "").unwrap();
    fs::write(src.join("Widget.tsx"), "").unwrap();
    fs::write(src.join("notes.md"), "").unwrap();

    let files = collect_lint_file_collection(&[src.display().to_string().into()], None).files;

    assert_eq!(
        files,
        vec![
            src.join("App.vue"),
            src.join("Panel.jsx"),
            src.join("Widget.tsx"),
            src.join("config.js"),
            src.join("index.html"),
            src.join("store.ts")
        ]
    );
}

#[test]
fn collection_applies_config_ignores_and_nested_node_modules() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src");
    let scripts_dep = dir.path().join("scripts/node_modules/chalk/source");
    fs::create_dir_all(&src).unwrap();
    fs::create_dir_all(&scripts_dep).unwrap();
    fs::write(src.join("App.vue"), "").unwrap();
    fs::write(src.join("Generated.vue"), "").unwrap();
    fs::write(scripts_dep.join("index.d.ts"), "").unwrap();

    let ignore_set = LintIgnoreSet::new(
        &[
            crate::config::ConfigEntryIgnore {
                base_path: None,
                pattern: "src/Generated.vue".into(),
            },
            crate::config::ConfigEntryIgnore {
                base_path: None,
                pattern: "node_modules/**".into(),
            },
        ],
        dir.path(),
    );
    let files = collect_lint_file_collection(
        &[dir.path().display().to_string().into()],
        ignore_set.as_ref(),
    )
    .files;

    assert_eq!(files, vec![src.join("App.vue")]);
}

#[test]
fn recursive_globs_include_dot_directories() {
    let dir = tempfile::tempdir().unwrap();
    let docs = dir.path().join("docs/.vitepress/components");
    fs::create_dir_all(&docs).unwrap();
    let download_page = docs.join("DownloadPage.vue");
    fs::write(&download_page, "").unwrap();

    let files = collect_lint_file_collection(
        &[dir.path().join("**/*.vue").display().to_string().into()],
        None,
    )
    .files;

    assert_eq!(files, vec![download_page]);
}

#[test]
fn collection_reports_each_pattern_with_no_lintable_matches() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src");
    fs::create_dir_all(&src).unwrap();
    let script = src.join("b.ts");
    fs::write(&script, "export const x = 1;\n").unwrap();

    let collection = collect_lint_file_collection(
        &[
            dir.path().join("src/**/*.ts").display().to_string().into(),
            dir.path()
                .join("src/**/*.{vue,ts}")
                .display()
                .to_string()
                .into(),
            dir.path().join("src/**/*.vue").display().to_string().into(),
        ],
        None,
    );

    assert_eq!(collection.files, vec![script]);
    assert_eq!(
        collection.unmatched_patterns,
        vec![
            vize_s0::String::from(dir.path().join("src/**/*.{vue,ts}").display().to_string()),
            vize_s0::String::from(dir.path().join("src/**/*.vue").display().to_string())
        ]
    );
}

#[test]
fn collection_does_not_report_overlapping_patterns_as_empty() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src");
    fs::create_dir_all(&src).unwrap();
    let script = src.join("b.ts");
    fs::write(&script, "export const x = 1;\n").unwrap();

    let collection = collect_lint_file_collection(
        &[
            src.display().to_string().into(),
            dir.path().join("src/**/*.ts").display().to_string().into(),
        ],
        None,
    );

    assert_eq!(collection.files, vec![script]);
    assert_eq!(collection.unmatched_patterns, Vec::<vize_s0::String>::new());
}

#[test]
fn collection_reports_ignored_only_patterns_as_empty() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("Generated.vue"), "").unwrap();

    let ignore_set = LintIgnoreSet::new(
        &[crate::config::ConfigEntryIgnore {
            base_path: None,
            pattern: "src/Generated.vue".into(),
        }],
        dir.path(),
    );
    let pattern = src.join("Generated.vue").display().to_string();
    let collection = collect_lint_file_collection(&[pattern.clone().into()], ignore_set.as_ref());

    assert_eq!(collection.files, Vec::<std::path::PathBuf>::new());
    assert_eq!(
        collection.unmatched_patterns,
        vec![vize_s0::String::from(pattern)]
    );
}
