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
