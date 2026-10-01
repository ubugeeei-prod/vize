use super::{FmtPattern, collect_files};
use crate::commands::fmt::ignores::FmtIgnoreSet;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use vize_l0::{String, ToCompactString};

#[test]
fn absolute_glob_only_matches_requested_directory() {
    let root = unique_case_dir("absolute-glob");
    let input_dir = root.join("bench-input");
    let other_dir = root.join("other");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(&other_dir).unwrap();
    fs::write(input_dir.join("A.vue"), "<template><div/></template>").unwrap();
    fs::write(other_dir.join("B.vue"), "<template><div/></template>").unwrap();

    let pattern = input_dir.join("*.vue").to_string_lossy().into_owned();
    let files = collect_files(&[pattern], None);
    let _ = fs::remove_dir_all(&root);

    assert_eq!(files, vec![input_dir.join("A.vue")]);
}

#[test]
fn collect_files_skips_generated_vize_workspace() {
    let root = unique_case_dir("generated-vize");
    let src = root.join("src");
    let generated = root.join("node_modules/.vize/corsa-overlay/src");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&src).unwrap();
    fs::create_dir_all(&generated).unwrap();
    fs::write(src.join("App.vue"), "<template><div/></template>").unwrap();
    fs::write(generated.join("App.vue"), "<template><div/></template>").unwrap();

    let pattern = root.join("**/*.vue").to_string_lossy().into_owned();
    let files = collect_files(&[pattern], None);
    let _ = fs::remove_dir_all(&root);

    assert_eq!(files, vec![src.join("App.vue")]);
}

#[test]
fn collect_files_matches_vue_scripts_and_jsx() {
    let root = unique_case_dir("format-targets");
    let src = root.join("src");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("App.vue"), "<template><div/></template>").unwrap();
    fs::write(src.join("config.js"), "export default {}").unwrap();
    fs::write(src.join("Panel.jsx"), "const Panel=()=> <div />").unwrap();
    fs::write(src.join("store.ts"), "export const count=0").unwrap();
    fs::write(src.join("Widget.tsx"), "const Widget=()=> <div />").unwrap();
    fs::write(src.join("types.d.ts"), "export type Widget = {}").unwrap();
    fs::write(src.join("package.json"), r#"{"name":"acme"}"#).unwrap();
    fs::write(src.join("config.yaml"), "a: 1\n").unwrap();
    fs::write(src.join("notes.md"), "# notes").unwrap();

    let pattern = root.to_string_lossy().into_owned();
    let files = collect_files(&[pattern], None);
    let _ = fs::remove_dir_all(&root);

    assert_eq!(
        files,
        vec![
            src.join("App.vue"),
            src.join("Panel.jsx"),
            src.join("Widget.tsx"),
            src.join("config.js"),
            src.join("config.yaml"),
            src.join("notes.md"),
            src.join("package.json"),
            src.join("store.ts"),
            src.join("types.d.ts"),
        ]
    );
}

#[test]
fn collect_files_applies_entry_ignores() {
    let root = unique_case_dir("entry-ignores");
    let src = root.join("src");
    let nested_node_modules = root.join("scripts/node_modules/chalk/source");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&src).unwrap();
    fs::create_dir_all(&nested_node_modules).unwrap();
    fs::write(src.join("App.vue"), "<template><div /></template>").unwrap();
    fs::write(src.join("Ignored.vue"), "<template><div /></template>").unwrap();
    fs::write(
        nested_node_modules.join("index.d.ts"),
        "export declare const chalk: string",
    )
    .unwrap();

    let ignore_set = FmtIgnoreSet::new(
        &[
            crate::config::ConfigEntryIgnore {
                base_path: None,
                pattern: "src/Ignored.vue".into(),
            },
            crate::config::ConfigEntryIgnore {
                base_path: None,
                pattern: "node_modules/**".into(),
            },
        ],
        &root,
    );
    let pattern = root.to_string_lossy().into_owned();
    let files = collect_files(&[pattern], ignore_set.as_ref());
    let explicit = collect_files(
        &[src.join("Ignored.vue").to_string_lossy().into_owned()],
        ignore_set.as_ref(),
    );
    let _ = fs::remove_dir_all(&root);

    assert_eq!(files, vec![src.join("App.vue")]);
    assert!(explicit.is_empty());
}

#[test]
fn relative_glob_does_not_match_every_vue_file() {
    let cwd = std::env::current_dir().unwrap();
    let pattern = FmtPattern::new("tools/benchmarks/scripts/__in__/*.vue", &cwd).unwrap();

    assert!(pattern.matches(Path::new(
        "./tools/benchmarks/scripts/__in__/Component0000.vue"
    )));
    assert!(!pattern.matches(Path::new("./examples/cli/src/App.vue")));
}

fn unique_case_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut dir_name = String::from(name);
    dir_name.push('-');
    let pid = std::process::id().to_compact_string();
    dir_name.push_str(pid.as_str());
    dir_name.push('-');
    let nanos = nanos.to_compact_string();
    dir_name.push_str(nanos.as_str());
    std::env::current_dir()
        .unwrap()
        .join("target")
        .join("vize-tests")
        .join("fmt")
        .join(dir_name.as_str())
}
