#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn temp_project_dir(test_name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "vize-build-inputs-cli-{}-{test_name}-{nonce}",
        std::process::id()
    ))
}

fn write_vue(root: &Path, relative_path: &str) {
    let path = root.join(relative_path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, "<template><div /></template>").unwrap();
}

fn run_build(root: &Path, inputs: &[&str], output: &str) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command
        .current_dir(root)
        .args(["build", "--format", "js", "--output", output])
        .args(inputs);
    command.output().unwrap()
}

#[test]
fn build_rejects_missing_relative_vue_literal_without_scanning_project() {
    let root = temp_project_dir("missing-relative-file");
    write_vue(&root, "src/Unrelated.vue");

    let input = "src/Missing.vue";
    let output = run_build(&root, &[input], "dist");

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        format!("Build input does not exist: {input}\n")
    );
    assert!(!root.join("dist").exists());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn build_rejects_missing_absolute_directory_literal_with_exact_input() {
    let root = temp_project_dir("missing-absolute-directory");
    fs::create_dir_all(&root).unwrap();
    let input = format!(
        "{}{}",
        root.join("missing-components").display(),
        std::path::MAIN_SEPARATOR
    );

    let output = run_build(&root, &[&input], "dist");

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        format!("Build input does not exist: {input}\n")
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn build_keeps_empty_glob_and_default_glob_diagnostics_distinct() {
    let root = temp_project_dir("glob-semantics");
    write_vue(&root, "src/Unrelated.vue");

    let empty_glob = "src/Missing*.vue";
    let empty = run_build(&root, &[empty_glob], "dist-empty");
    assert!(!empty.status.success());
    assert_eq!(
        String::from_utf8_lossy(&empty.stderr),
        "No .vue files found matching the patterns\n"
    );

    let default = run_build(&root, &[], "dist-default");
    assert!(
        default.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&default.stdout),
        String::from_utf8_lossy(&default.stderr)
    );
    assert!(root.join("dist-default/src/Unrelated.js").is_file());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn build_accepts_relative_files_and_absolute_directories() {
    let root = temp_project_dir("valid-literals");
    write_vue(&root, "src/App.vue");

    let relative = run_build(&root, &["src/App.vue"], "dist-relative");
    assert!(
        relative.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&relative.stderr)
    );
    assert!(root.join("dist-relative/App.js").is_file());

    let absolute_directory = root.join("src").display().to_string();
    let absolute = run_build(&root, &[&absolute_directory], "dist-absolute");
    assert!(
        absolute.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&absolute.stderr)
    );
    assert!(root.join("dist-absolute/App.js").is_file());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn build_rejects_existing_non_vue_literal_before_writing_other_inputs() {
    let root = temp_project_dir("non-vue-file");
    write_vue(&root, "src/App.vue");
    fs::write(root.join("README.md"), "not an SFC").unwrap();

    let output = run_build(&root, &["src/App.vue", "README.md"], "dist");

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "Build input is not a .vue file: README.md\n"
    );
    assert!(!root.join("dist").exists());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn build_honors_segment_and_recursive_glob_syntax() {
    let root = temp_project_dir("general-globs");
    for file in [
        "src/App1.vue",
        "src/App12.vue",
        "src/Button.vue",
        "src/Card.vue",
        "src/nested/App2.vue",
        "src/nested/deep/App3.vue",
    ] {
        write_vue(&root, file);
    }

    let question = run_build(&root, &["src/App?.vue"], "dist-question");
    assert_success(&question);
    assert!(root.join("dist-question/App1.js").is_file());
    assert!(!root.join("dist-question/App12.js").exists());

    let class = run_build(&root, &["src/[AB]*.vue"], "dist-class");
    assert_success(&class);
    assert!(root.join("dist-class/App1.js").is_file());
    assert!(root.join("dist-class/Button.js").is_file());
    assert!(!root.join("dist-class/Card.js").exists());

    let segment = run_build(&root, &["src/*/App?.vue"], "dist-segment");
    assert_success(&segment);
    assert!(root.join("dist-segment/nested/App2.js").is_file());
    assert!(!root.join("dist-segment/nested/deep/App3.js").exists());

    let recursive = run_build(&root, &["src/**/App?.vue"], "dist-recursive");
    assert_success(&recursive);
    assert!(root.join("dist-recursive/App1.js").is_file());
    assert!(root.join("dist-recursive/nested/App2.js").is_file());
    assert!(root.join("dist-recursive/nested/deep/App3.js").is_file());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn build_honors_absolute_globs_and_source_relative_outputs() {
    let root = temp_project_dir("absolute-glob");
    write_vue(&root, "src/App1.vue");
    write_vue(&root, "src/nested/App2.vue");
    write_vue(&root, "other/App3.vue");
    let pattern = root.join("src/**/App?.vue").display().to_string();

    let output = run_build(&root, &[&pattern], "dist");

    assert_success(&output);
    assert!(root.join("dist/App1.js").is_file());
    assert!(root.join("dist/nested/App2.js").is_file());
    assert!(!root.join("dist/other/App3.js").exists());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn build_reports_invalid_glob_syntax_exactly_before_writing() {
    let root = temp_project_dir("invalid-glob");
    write_vue(&root, "src/App.vue");
    let pattern = "src/[AB.vue";

    let output = run_build(&root, &[pattern], "dist");

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        format!(
            "Invalid glob pattern {pattern}: Pattern syntax error near position 4: invalid range pattern\n"
        )
    );
    assert!(!root.join("dist").exists());

    let _ = fs::remove_dir_all(root);
}

#[cfg(not(windows))]
#[test]
fn build_uses_bracket_expressions_to_escape_metacharacters() {
    let root = temp_project_dir("escaped-glob");
    write_vue(&root, "src/Component[old]-*-?.vue");
    write_vue(&root, "src/Component-new.vue");
    let pattern = "src/Component[[]old[]]-[*]-[?].vue";

    let output = run_build(&root, &[pattern], "dist");

    assert_success(&output);
    assert!(root.join("dist/Component[old]-*-?.js").is_file());
    assert!(!root.join("dist/Component-new.js").exists());

    let _ = fs::remove_dir_all(root);
}

#[cfg(windows)]
#[test]
fn build_accepts_native_windows_globs_case_insensitively() {
    let root = temp_project_dir("windows-glob");
    write_vue(&root, "src/CardA.vue");
    let pattern = root.join("src/card?.vue").display().to_string();

    let output = run_build(&root, &[&pattern], "dist");

    assert_success(&output);
    assert!(root.join("dist/CardA.js").is_file());

    let _ = fs::remove_dir_all(root);
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
