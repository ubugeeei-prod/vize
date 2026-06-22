use std::{fs, path::Path, process::Command};

fn write_project_file(root: &Path, path: &str, content: &str) {
    let file_path = root.join(path);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(file_path, content).unwrap();
}

#[test]
fn lint_storybook_tsx_csf_bypasses_template_rules() {
    let project = tempfile::tempdir().unwrap();
    write_project_file(
        project.path(),
        "src/AfButton.stories.tsx",
        r#"const items = [{ id: 1, label: "Open" }];

export const Example = () => (
  <div>
    {items.map((item) => (
      <button onClick={(event) => event.preventDefault()} isOpened>
        {item.label}
      </button>
    ))}
  </div>
);
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args([
            "lint",
            "--no-config",
            "--preset",
            "happy-path",
            "--format",
            "json",
            "src/AfButton.stories.tsx",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(!stdout.contains("vue/require-v-for-key"), "{stdout}");
    assert!(!stdout.contains("vue/prop-name-casing"), "{stdout}");
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let [file] = json.as_array().unwrap().as_slice() else {
        panic!("expected one lint result, got {stdout}");
    };
    assert_eq!(file["errorCount"], 0, "{stdout}");
    assert_eq!(file["warningCount"], 0, "{stdout}");
    assert_eq!(file["messages"], serde_json::json!([]), "{stdout}");
}
