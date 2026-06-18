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

fn write_entry_config(root: &Path) {
    write_project_file(
        root,
        "vize.config.json",
        r#"{
  "entries": [
    {
      "name": "design-system",
      "basePath": "design-system",
      "files": ["src/**/*.vue", "src/**/*.art.vue"]
    }
  ]
}"#,
    );
}

#[test]
fn fmt_check_supports_root_relative_paths_for_config_entries() {
    let project = tempfile::tempdir().unwrap();
    write_entry_config(project.path());
    write_project_file(
        project.path(),
        "design-system/src/AfsButton.vue",
        "<template><div>{{msg}}</div></template>\n<script setup lang=\"ts\">const msg=1</script>\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args([
            "fmt",
            "--config",
            "vize.config.json",
            "--check",
            "design-system/src/AfsButton.vue",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1), "{}", output_details(&output));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Found 1 file(s)"));
    assert!(stderr.contains("Would reformat: design-system/src/AfsButton.vue"));
    assert!(!stderr.contains("No .vue"));
}

#[test]
fn fmt_check_fails_when_explicit_patterns_match_no_files() {
    let project = tempfile::tempdir().unwrap();
    write_entry_config(project.path());

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args([
            "fmt",
            "--config",
            "vize.config.json",
            "--check",
            "design-system/src/**/*.art.vue",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1), "{}", output_details(&output));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No .vue, .js, .ts, .jsx, or .tsx files found"));
}
