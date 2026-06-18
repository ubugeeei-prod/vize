use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn write_project_file(root: &Path, path: &str, content: &str) {
    let file_path = root.join(path);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(file_path, content).unwrap();
}

fn output_details(output: &Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn fmt_check_supports_plain_ts_inputs() {
    let project = tempfile::tempdir().unwrap();
    write_project_file(project.path(), "vite.config.ts", "export default {foo:1}\n");

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args(["fmt", "--no-config", "--check", "vite.config.ts"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1), "{}", output_details(&output));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Found 1 file(s)"));
    assert!(stderr.contains("Would reformat: vite.config.ts"));
    assert!(stderr.contains("Checked 1 file(s)"));
    assert!(!stderr.contains("No .vue"));
}

#[test]
fn lint_supports_plain_ts_inputs() {
    let project = tempfile::tempdir().unwrap();
    write_project_file(
        project.path(),
        "vite.config.ts",
        r#"import { getCurrentInstance } from "vue";

const instance = getCurrentInstance();
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args([
            "lint",
            "--no-config",
            "--preset",
            "opinionated",
            "--format",
            "text",
            "--help-level",
            "none",
            "vite.config.ts",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1), "{}", output_details(&output));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("script/no-get-current-instance"));
    assert!(stdout.contains("Linted 1 files"));
}
