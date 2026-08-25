#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use super::{BuildInput, classify_input, collect_files, contains_glob_metacharacter};
use std::{
    fs,
    path::{Path, PathBuf},
};
use vize_s0::ToCompactString;

#[test]
fn collect_files_ignores_vue_extension_directories() {
    let root = unique_case_dir("build-vue-extension-directories");
    let src = root.join("src");
    let component_dir = src.join("Directory.vue");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&component_dir).unwrap();
    fs::write(src.join("App.vue"), "<template><div /></template>").unwrap();
    fs::write(
        component_dir.join("Nested.vue"),
        "<template><div /></template>",
    )
    .unwrap();

    let collected = collect_files(&[root.display().to_string()]).unwrap();
    let _ = fs::remove_dir_all(&root);

    let mut expected = vec![component_dir.join("Nested.vue"), src.join("App.vue")];
    expected.sort();
    assert_eq!(collected.files, expected);
    assert_eq!(collected.roots, vec![root]);
}

#[test]
fn collect_files_keeps_direct_vue_file_patterns() {
    let root = unique_case_dir("build-direct-vue-file");
    let src = root.join("src");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&src).unwrap();
    let app = src.join("App.vue");
    fs::write(&app, "<template><div /></template>").unwrap();
    fs::write(src.join("Sibling.vue"), "<template><div /></template>").unwrap();

    let collected = collect_files(&[app.display().to_string()]).unwrap();
    let _ = fs::remove_dir_all(&root);

    assert_eq!(collected.files, vec![app]);
    assert_eq!(collected.roots, vec![src]);
}

#[test]
fn collect_files_keeps_empty_searched_roots() {
    let root = unique_case_dir("build-empty-searched-root");
    let alpha = root.join("packages/alpha/src");
    let beta = root.join("packages/beta/src");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&alpha).unwrap();
    fs::create_dir_all(&beta).unwrap();
    let app = alpha.join("App.vue");
    fs::write(&app, "<template><div /></template>").unwrap();

    let collected = collect_files(&[
        alpha.to_string_lossy().into_owned(),
        beta.to_string_lossy().into_owned(),
    ])
    .unwrap();
    let _ = fs::remove_dir_all(&root);

    assert_eq!(collected.files, vec![app]);
    assert_eq!(collected.roots, vec![alpha, beta]);
}

#[test]
fn collect_files_rejects_missing_literals_but_accepts_empty_globs() {
    let root = unique_case_dir("build-missing-literal");
    let missing_literal = root.join("Missing.vue").display().to_string();
    let error = collect_files(std::slice::from_ref(&missing_literal)).unwrap_err();

    assert_eq!(
        error.to_string(),
        format!("Build input does not exist: {missing_literal}")
    );

    let empty_glob = root.join("missing/**/*.vue").display().to_string();
    assert!(
        collect_files(std::slice::from_ref(&empty_glob))
            .unwrap()
            .files
            .is_empty()
    );
}

#[test]
fn collect_files_rejects_non_vue_files_before_collecting_roots() {
    let root = unique_case_dir("build-non-vue-literal");
    let app = root.join("src/App.vue");
    let readme = root.join("README.md");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(app.parent().unwrap()).unwrap();
    fs::write(&app, "<template><div /></template>").unwrap();
    fs::write(&readme, "not an SFC").unwrap();
    let inputs = [app.display().to_string(), readme.display().to_string()];

    let error = collect_files(&inputs).unwrap_err();
    let _ = fs::remove_dir_all(&root);

    assert_eq!(
        error.to_string(),
        format!("Build input is not a .vue file: {}", readme.display())
    );
}

#[test]
fn collect_files_accepts_backslash_pattern_separators() {
    let root = unique_case_dir("build-backslash-glob");
    let app = root.join("src/App.vue");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(app.parent().unwrap()).unwrap();
    fs::write(&app, "<template><div /></template>").unwrap();
    let pattern = root.join("src/*.vue").to_string_lossy().replace('/', "\\");

    let collected = collect_files(&[pattern]).unwrap();
    let _ = fs::remove_dir_all(&root);

    assert_eq!(collected.files, vec![app]);
}

#[test]
fn existing_paths_with_metacharacters_remain_literals() {
    let root = unique_case_dir("build-literal-metacharacter");
    let file = root.join("Component[old].vue");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    fs::write(&file, "<template><div /></template>").unwrap();

    assert!(matches!(
        classify_input(file.to_str().unwrap()).unwrap(),
        BuildInput::File(path) if path == file
    ));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn classifies_windows_separators_and_glob_metacharacters_portably() {
    let pattern = r"C:\workspace\src\**\*.vue";
    let BuildInput::Glob(classified) = classify_input(pattern).unwrap() else {
        panic!("Windows-style glob should be classified as a glob");
    };

    let expected_root = if cfg!(windows) {
        r"C:\workspace\src\"
    } else {
        "C:/workspace/src/"
    };
    assert_eq!(classified.root().to_string_lossy(), expected_root);
    assert!(contains_glob_metacharacter(r"src\?.vue"));
    assert!(contains_glob_metacharacter(r"src\[AB].vue"));
    assert!(!contains_glob_metacharacter(r"C:\workspace\src\App.vue"));
    assert!(classified.matches(Path::new(r"C:\workspace\src\App.vue")));
}

fn unique_case_dir(name: &str) -> PathBuf {
    static NEXT_CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("vize-tests")
        .join(format!(
            "{}-{}-{}",
            name,
            std::process::id(),
            case_id.to_compact_string()
        ))
}
