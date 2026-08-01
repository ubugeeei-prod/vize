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

#[test]
fn quiet_lint_reduces_warning_totals_and_preserves_max_warnings_exit() {
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
    let source = r#"<script setup lang="ts">
const emit = defineEmits(["update:current-step-index"])
emit("update:current-step-index", 1)
</script>
"#;
    write_file(project_root.path(), "src/First.vue", source);
    write_file(project_root.path(), "src/Second.vue", source);

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root.path())
        .args([
            "lint",
            "--quiet",
            "--config",
            "vize.config.json",
            "--max-warnings",
            "0",
            "src/*.vue",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1), "{}", output_details(&output));
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    assert!(stdout.contains("2 warnings in 2 files"), "{stdout}");
    assert!(!stdout.contains("custom-event-name-casing"), "{stdout}");
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert!(stderr.contains("Too many warnings (2 > max 0)"), "{stderr}");
}

#[test]
fn lint_applies_entry_rule_severity_only_to_matching_files() {
    let project_root = tempfile::tempdir().unwrap();
    write_file(
        project_root.path(),
        "vize.config.json",
        r#"{
  "linter": {
    "preset": "ecosystem",
    "rules": { "script/custom-event-name-casing": "error" }
  },
  "entries": [{
    "files": ["src/pages/**/*.vue"],
    "ignores": ["src/pages/ignored.vue"],
    "linter": { "rules": { "script/custom-event-name-casing": "warn" } }
  }]
}"#,
    );
    let source = r#"<script setup lang="ts">
const emit = defineEmits(["update:current-step-index"])
emit("update:current-step-index", 1)
</script>
"#;
    write_file(project_root.path(), "src/components/Card.vue", source);
    write_file(project_root.path(), "src/pages/ignored.vue", source);
    write_file(project_root.path(), "src/pages/index.vue", source);

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root.path())
        .args([
            "lint",
            "--config",
            "vize.config.json",
            "--format",
            "json",
            "src/**/*.vue",
        ])
        .output()
        .unwrap();

    let actual = serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();
    assert_eq!(
        actual,
        serde_json::json!([
          {
            "file": "src/components/Card.vue",
            "messages": [{
              "ruleId": "script/custom-event-name-casing",
              "ruleDocsPath": "docs/content/rules/type-and-script.md",
              "severity": 2,
              "message": "[vize:script/custom-event-name-casing] Custom event name 'update:current-step-index' is not camelCase.",
              "line": 3,
              "column": 6,
              "endLine": 3,
              "endColumn": 33,
              "help": "Vue 3 recommends camelCase for emitted event names; rename this event to camelCase (e.g. myEvent)."
            }],
            "errorCount": 1,
            "warningCount": 0
          },
          {
            "file": "src/pages/ignored.vue",
            "messages": [{
              "ruleId": "script/custom-event-name-casing",
              "ruleDocsPath": "docs/content/rules/type-and-script.md",
              "severity": 2,
              "message": "[vize:script/custom-event-name-casing] Custom event name 'update:current-step-index' is not camelCase.",
              "line": 3,
              "column": 6,
              "endLine": 3,
              "endColumn": 33,
              "help": "Vue 3 recommends camelCase for emitted event names; rename this event to camelCase (e.g. myEvent)."
            }],
            "errorCount": 1,
            "warningCount": 0
          },
          {
            "file": "src/pages/index.vue",
            "messages": [{
              "ruleId": "script/custom-event-name-casing",
              "ruleDocsPath": "docs/content/rules/type-and-script.md",
              "severity": 1,
              "message": "[vize:script/custom-event-name-casing] Custom event name 'update:current-step-index' is not camelCase.",
              "line": 3,
              "column": 6,
              "endLine": 3,
              "endColumn": 33,
              "help": "Vue 3 recommends camelCase for emitted event names; rename this event to camelCase (e.g. myEvent)."
            }],
            "errorCount": 0,
            "warningCount": 1
          }
        ]),
        "{}",
        output_details(&output),
    );
}
