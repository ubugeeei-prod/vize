#![allow(clippy::disallowed_macros, clippy::disallowed_types)]

use std::{fs, path::Path, process::Command};

fn write_project_file(root: &Path, path: &str, content: &str) {
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
fn lint_runs_explicit_musea_rules_for_art_files() {
    let project_root = tempfile::tempdir().unwrap();
    write_project_file(
        project_root.path(),
        "src/Button.art.vue",
        r#"<art component="./Button.vue">
  <variant name="empty"></variant>
</art>
"#,
    );

    let default_output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root.path())
        .args([
            "lint",
            "--no-config",
            "--format",
            "json",
            "src/Button.art.vue",
        ])
        .output()
        .unwrap();
    assert!(
        default_output.status.success(),
        "{}",
        output_details(&default_output)
    );
    let default_json: serde_json::Value = serde_json::from_slice(&default_output.stdout)
        .unwrap_or_else(|_| {
            panic!(
                "default lint must emit JSON: {}",
                output_details(&default_output)
            )
        });
    assert_eq!(
        default_json,
        serde_json::json!([{
            "file": "src/Button.art.vue",
            "messages": [],
            "errorCount": 0,
            "warningCount": 0
        }]),
        "{}",
        output_details(&default_output)
    );

    write_project_file(
        project_root.path(),
        "vize.config.json",
        r#"{
  "linter": {
    "preset": "incremental",
    "rules": {
      "musea/require-title": "error",
      "musea/no-empty-variant": "warn"
    }
  }
}"#,
    );

    let configured_output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root.path())
        .args([
            "lint",
            "--config",
            "vize.config.json",
            "--format",
            "json",
            "--max-warnings",
            "0",
            "src/Button.art.vue",
        ])
        .output()
        .unwrap();
    assert!(
        !configured_output.status.success(),
        "{}",
        output_details(&configured_output)
    );
    let configured_json: serde_json::Value = serde_json::from_slice(&configured_output.stdout)
        .unwrap_or_else(|_| {
            panic!(
                "configured lint must emit JSON: {}",
                output_details(&configured_output)
            )
        });
    assert_eq!(
        configured_json,
        serde_json::json!([{
            "file": "src/Button.art.vue",
            "messages": [
                {
                    "ruleId": "musea/require-title",
                    "ruleDocsPath": "docs/content/rules/musea-and-css.md",
                    "severity": 2,
                    "message": "[vize:musea/require-title] Missing required 'title' attribute in <art> block",
                    "line": 1,
                    "column": 1,
                    "endLine": 1,
                    "endColumn": 30,
                    "help": "Add a title attribute: <art title=\"Component Name\">"
                },
                {
                    "ruleId": "musea/no-empty-variant",
                    "ruleDocsPath": "docs/content/rules/musea-and-css.md",
                    "severity": 1,
                    "message": "[vize:musea/no-empty-variant] Empty <variant> block with no content",
                    "line": 2,
                    "column": 3,
                    "endLine": 2,
                    "endColumn": 35,
                    "help": "Add template content inside the variant"
                }
            ],
            "errorCount": 1,
            "warningCount": 1
        }]),
        "{}",
        output_details(&configured_output)
    );
}

#[test]
fn lint_runs_configured_musea_design_token_rule_for_art_styles() {
    let project_root = tempfile::tempdir().unwrap();
    write_project_file(
        project_root.path(),
        "vize.config.json",
        r##"{
  "linter": {
    "preset": "incremental",
    "ruleOptions": {
      "musea/prefer-design-tokens": {
        "tokens": [
          {
            "path": "color.primary",
            "value": "#3b82f6"
          }
        ]
      }
    }
  }
}"##,
    );
    write_project_file(
        project_root.path(),
        "src/Button.art.vue",
        r##"<art title="Button" component="./Button.vue">
  <variant name="default"><button class="button">Save</button></variant>
</art>

<style scoped>
.button {
  background: #3b82f6;
}
</style>
"##,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root.path())
        .args([
            "lint",
            "--config",
            "vize.config.json",
            "--format",
            "json",
            "--max-warnings",
            "0",
            "src/Button.art.vue",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success(), "{}", output_details(&output));
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|_| {
        panic!(
            "configured lint must emit JSON: {}",
            output_details(&output)
        )
    });
    assert_eq!(
        json,
        serde_json::json!([{
            "file": "src/Button.art.vue",
            "messages": [
                {
                    "ruleId": "musea/prefer-design-tokens",
                    "ruleDocsPath": "docs/content/rules/musea-and-css.md",
                    "severity": 1,
                    "message": "[vize:musea/prefer-design-tokens] Hardcoded value '#3b82f6' matches primitive token 'color.primary' — use var(--color-primary)",
                    "line": 7,
                    "column": 1,
                    "endLine": 7,
                    "endColumn": 23,
                    "help": "Use var(--color-primary) for consistent theming and maintainability"
                }
            ],
            "errorCount": 0,
            "warningCount": 1
        }]),
        "{}",
        output_details(&output)
    );
}
