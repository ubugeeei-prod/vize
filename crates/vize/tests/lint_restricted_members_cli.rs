#![allow(clippy::disallowed_macros, clippy::disallowed_types)]

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

fn write_restricted_member_sfc(root: &Path) {
    write_file(
        root,
        "src/App.vue",
        r#"<script setup lang="ts">
const token = window.localStorage.getItem("auth.token")
</script>
"#,
    );
}

#[test]
fn configured_restricted_members_enable_the_project_local_rule() {
    let project_root = tempfile::tempdir().unwrap();
    write_file(
        project_root.path(),
        "vize.config.json",
        r#"{
  "linter": {
    "preset": "incremental",
    "ruleOptions": {
      "script/no-restricted-members": {
        "members": [
          {
            "object": "window",
            "property": "localStorage",
            "message": "Use authStorage."
          }
        ]
      }
    }
  }
}"#,
    );
    write_restricted_member_sfc(project_root.path());

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
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

    assert_eq!(output.status.code(), Some(1), "{}", output_details(&output));
    let actual = serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();
    assert_eq!(
        actual,
        serde_json::json!([{
          "file": "src/App.vue",
          "messages": [{
            "ruleId": "script/no-restricted-members",
            "ruleDocsPath": "docs/content/rules/type-and-script.md",
            "severity": 2,
            "message": "[vize:script/no-restricted-members] Use authStorage.",
            "line": 2,
            "column": 15,
            "endLine": 2,
            "endColumn": 34
          }],
          "errorCount": 1,
          "warningCount": 0
        }]),
        "{}",
        output_details(&output),
    );
}

#[test]
fn explicit_off_keeps_configured_restricted_members_disabled() {
    let project_root = tempfile::tempdir().unwrap();
    write_file(
        project_root.path(),
        "vize.config.json",
        r#"{
  "linter": {
    "preset": "incremental",
    "rules": {
      "script/no-restricted-members": "off"
    },
    "ruleOptions": {
      "script/no-restricted-members": {
        "members": [
          {
            "object": "window",
            "property": "localStorage",
            "message": "Use authStorage."
          }
        ]
      }
    }
  }
}"#,
    );
    write_restricted_member_sfc(project_root.path());

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
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

    assert!(output.status.success(), "{}", output_details(&output));
    let actual = serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();
    assert_eq!(
        actual,
        serde_json::json!([{
          "file": "src/App.vue",
          "messages": [],
          "errorCount": 0,
          "warningCount": 0
        }]),
        "{}",
        output_details(&output),
    );
}
