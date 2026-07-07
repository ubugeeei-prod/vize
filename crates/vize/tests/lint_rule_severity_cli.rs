use std::{fs, path::Path, process::Command};

fn write_file(root: &Path, path: &str, content: &str) {
    let file_path = root.join(path);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(file_path, content).unwrap();
}

fn output_details(output: &std::process::Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn lint_config_rule_severity_override_applies_to_builtin_script_rules() {
    let project_root = tempfile::tempdir().unwrap();
    write_file(
        project_root.path(),
        "vize.config.json",
        r#"{
  "linter": {
    "preset": "ecosystem",
    "rules": {
      "script/custom-event-name-casing": "warn"
    }
  }
}"#,
    );
    write_file(
        project_root.path(),
        "src/AfsStepperDialog.vue",
        r#"<script setup lang="ts">
const emit = defineEmits(["update:current-step-index"])

const handleStepClick = (stepIndex: number) => {
  emit("update:current-step-index", stepIndex)
}
</script>
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root.path())
        .args([
            "lint",
            "--config",
            "vize.config.json",
            "--format",
            "json",
            "src/AfsStepperDialog.vue",
        ])
        .output()
        .unwrap();

    assert!(output.status.success(), "{}", output_details(&output));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let file = json.as_array().unwrap().first().unwrap();
    assert_eq!(file["errorCount"], serde_json::json!(0), "{stdout}");
    assert_eq!(file["warningCount"], serde_json::json!(1), "{stdout}");
    assert_eq!(
        file["messages"][0]["ruleId"],
        serde_json::json!("script/custom-event-name-casing"),
        "{stdout}"
    );
    assert_eq!(
        file["messages"][0]["severity"],
        serde_json::json!(1),
        "{stdout}"
    );
}
