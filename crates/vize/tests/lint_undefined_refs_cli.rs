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
fn lint_config_can_enable_undefined_template_refs() {
    let project_root = tempfile::tempdir().unwrap();
    write_file(
        project_root.path(),
        "vize.config.json",
        r#"{
  "linter": {
    "rules": {
      "vue/no-undefined-refs": "warn"
    }
  }
}"#,
    );
    write_file(
        project_root.path(),
        "src/App.vue",
        r#"<script setup>
const known = 1
</script>
<template>
  <p>{{ known }} {{ missing }}</p>
</template>
"#,
    );

    let enabled = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root.path())
        .args([
            "lint",
            "--config",
            "vize.config.json",
            "--format",
            "json",
            "src/App.vue",
        ])
        .output()
        .unwrap();
    assert!(
        enabled.status.success(),
        "config-enable must keep the warning exit: {}",
        output_details(&enabled)
    );
    let enabled_json: serde_json::Value = serde_json::from_slice(&enabled.stdout)
        .unwrap_or_else(|_| panic!("config-enable must emit JSON: {}", output_details(&enabled)));
    assert_eq!(
        enabled_json,
        serde_json::json!([{
            "file": "src/App.vue",
            "messages": [{
                "ruleId": "vue/no-undefined-refs",
                "ruleDocsPath": "docs/content/rules/vue.md",
                "severity": 1,
                "message": "[vize:vue/no-undefined-refs] Variable 'missing' is not defined",
                "line": 5,
                "column": 21,
                "endLine": 5,
                "endColumn": 28,
                "help": "Define in <script setup> or ensure it's imported"
            }],
            "errorCount": 0,
            "warningCount": 1
        }]),
        "{}",
        output_details(&enabled)
    );

    let default = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root.path())
        .args(["lint", "--no-config", "--format", "json", "src/App.vue"])
        .output()
        .unwrap();
    assert!(
        default.status.success(),
        "default preset must stay silent: {}",
        output_details(&default)
    );
    let default_json: serde_json::Value =
        serde_json::from_slice(&default.stdout).unwrap_or_else(|_| {
            panic!(
                "default preset must emit JSON: {}",
                output_details(&default)
            )
        });
    assert_eq!(
        default_json,
        serde_json::json!([{
            "file": "src/App.vue",
            "messages": [],
            "errorCount": 0,
            "warningCount": 0
        }]),
        "{}",
        output_details(&default)
    );
}
