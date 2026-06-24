//! `vize fmt` applies conservative YAML/Markdown normalization end to end.

use std::{fs, path::Path, process::Command};

fn write_project_file(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn run_fmt(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(dir)
        .args(["fmt"].iter().chain(args).copied().collect::<Vec<_>>())
        .output()
        .unwrap()
}

#[test]
fn fmt_normalizes_yaml_line_endings_and_final_newline() {
    let project = tempfile::tempdir().unwrap();
    write_project_file(project.path(), "config.yaml", "a: 1\r\nb: 2");

    let output = run_fmt(project.path(), &["--write", "config.yaml"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(0), "stderr: {stderr}");

    let contents = fs::read_to_string(project.path().join("config.yaml")).unwrap();
    assert_eq!(contents, "a: 1\nb: 2\n");
}

#[test]
fn fmt_trims_markdown_trailing_whitespace_outside_code() {
    let project = tempfile::tempdir().unwrap();
    write_project_file(
        project.path(),
        "README.md",
        "# Title   \n\n```sh\necho hi   \n```\ntext \n",
    );

    let output = run_fmt(project.path(), &["--write", "README.md"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(0), "stderr: {stderr}");

    let contents = fs::read_to_string(project.path().join("README.md")).unwrap();
    // Heading + text trimmed; trailing spaces inside the fence preserved.
    assert_eq!(contents, "# Title\n\n```sh\necho hi   \n```\ntext\n");
}

#[test]
fn fmt_check_passes_on_already_normalized_yaml() {
    let project = tempfile::tempdir().unwrap();
    write_project_file(project.path(), "ok.yaml", "name: vize\n");

    let output = run_fmt(project.path(), &["--check", "ok.yaml"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(0), "stderr: {stderr}");
    assert!(
        stderr.contains("1 file(s) already formatted"),
        "expected already-formatted summary, got: {stderr}",
    );
}
