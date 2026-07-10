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
fn fmt_check_supports_root_relative_json_paths_for_config_entries() {
    let project = tempfile::tempdir().unwrap();
    write_project_file(
        project.path(),
        "vize.config.json",
        r#"{
  "entries": [
    {
      "name": "design-system",
      "basePath": "design-system",
      "files": ["src/**/*.json", "src/**/*.md"]
    }
  ]
}"#,
    );
    write_project_file(
        project.path(),
        "design-system/src/package.json",
        r#"{"name":"acme"}"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args([
            "fmt",
            "--config",
            "vize.config.json",
            "--check",
            "src/package.json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1), "{}", output_details(&output));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Found 1 file(s)"), "{stderr}");
    assert!(
        stderr.contains("Would reformat: design-system/src/package.json"),
        "{stderr}"
    );
    assert!(!stderr.contains("No .vue"), "{stderr}");
}
