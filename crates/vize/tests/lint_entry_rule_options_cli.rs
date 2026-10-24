#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use crate::support::vite_config;
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

fn write_session_storage_sfc(root: &Path, path: &str) {
    write_file(
        root,
        path,
        r#"<script setup lang="ts">
const token = window.sessionStorage.getItem("auth.token")
</script>
"#,
    );
}

#[test]
fn entry_rule_options_enable_project_local_rules_for_matching_files() {
    let project_root = tempfile::tempdir().unwrap();
    write_file(
        project_root.path(),
        "vite.config.mjs",
        &vite_config::source(
            r#"{
  "linter": {
    "preset": "incremental"
  },
  "entries": [
    {
      "files": ["src/admin/**/*.vue"],
      "linter": {
        "ruleOptions": {
          "script/no-restricted-members": {
            "members": [
              {
                "object": "window",
                "property": "sessionStorage",
                "message": "Use adminStorage."
              }
            ]
          }
        }
      }
    }
  ]
}"#,
        ),
    );
    write_session_storage_sfc(project_root.path(), "src/admin/App.vue");
    write_session_storage_sfc(project_root.path(), "src/public/App.vue");

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root.path())
        .args([
            "lint",
            "--config",
            "vite.config.mjs",
            "--format",
            "json",
            "src/admin/App.vue",
            "src/public/App.vue",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1), "{}", output_details(&output));
    let actual = serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();
    assert_eq!(
        actual,
        serde_json::json!([
          {
            "file": "src/admin/App.vue",
            "messages": [{
              "ruleId": "script/no-restricted-members",
              "ruleDocsPath": "docs/content/rules/type-and-script.md",
              "severity": 2,
              "message": "[vize:script/no-restricted-members] Use adminStorage.",
              "line": 2,
              "column": 15,
              "endLine": 2,
              "endColumn": 36
            }],
            "errorCount": 1,
            "warningCount": 0
          },
          {
            "file": "src/public/App.vue",
            "messages": [],
            "errorCount": 0,
            "warningCount": 0
          }
        ]),
        "{}",
        output_details(&output),
    );
}

#[test]
fn define_macros_order_accepts_define_options_first_from_vite_config() {
    let project_root = tempfile::tempdir().unwrap();
    write_file(
        project_root.path(),
        "vite.config.mjs",
        &vite_config::source(
            r#"{
  "linter": {
    "preset": "incremental",
    "rules": { "script/define-macros-order": "error" },
    "ruleOptions": {
      "script/define-macros-order": {
        "order": ["defineOptions", "defineProps", "defineEmits", "defineSlots"]
      }
    }
  }
}"#,
        ),
    );
    write_file(
        project_root.path(),
        "src/MyInput.vue",
        r#"<script setup lang="ts">
defineOptions({ inheritAttrs: false })
const { label } = defineProps<{ label: string }>()
const emit = defineEmits<{ select: [] }>()
defineSlots<{ default(): unknown }>()
</script>
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root.path())
        .args([
            "lint",
            "--config",
            "vite.config.mjs",
            "--format",
            "json",
            "src/MyInput.vue",
        ])
        .output()
        .unwrap();

    assert!(output.status.success(), "{}", output_details(&output));
    let actual = serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();
    assert_eq!(
        actual,
        serde_json::json!([{
            "file": "src/MyInput.vue",
            "messages": [],
            "errorCount": 0,
            "warningCount": 0
        }]),
        "{}",
        output_details(&output),
    );
}
