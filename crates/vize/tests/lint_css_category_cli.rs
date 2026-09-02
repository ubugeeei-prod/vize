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

fn write_button_sfc(root: &Path) {
    write_file(
        root,
        "src/Button.vue",
        r#"<style scoped>
.button { color: red !important; }
</style>
"#,
    );
}

#[test]
fn lint_config_style_category_disables_builtin_css_rules() {
    let project_root = tempfile::tempdir().unwrap();
    write_file(
        project_root.path(),
        "vize.config.json",
        r#"{
  "linter": {
    "preset": "opinionated",
    "categories": {
      "style": "off"
    }
  }
}"#,
    );
    write_button_sfc(project_root.path());

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root.path())
        .args([
            "lint",
            "--config",
            "vize.config.json",
            "--format",
            "json",
            "src/Button.vue",
        ])
        .output()
        .unwrap();

    assert!(output.status.success(), "{}", output_details(&output));
    let actual = serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();
    assert_eq!(
        actual,
        serde_json::json!([{
          "file": "src/Button.vue",
          "messages": [],
          "errorCount": 0,
          "warningCount": 0
        }]),
        "{}",
        output_details(&output),
    );
}

#[test]
fn lint_config_style_category_severity_applies_to_builtin_css_rules() {
    let project_root = tempfile::tempdir().unwrap();
    write_file(
        project_root.path(),
        "vize.config.json",
        r#"{
  "linter": {
    "preset": "opinionated",
    "categories": {
      "style": "error"
    }
  }
}"#,
    );
    write_button_sfc(project_root.path());

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root.path())
        .args([
            "lint",
            "--config",
            "vize.config.json",
            "--format",
            "json",
            "src/Button.vue",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1), "{}", output_details(&output));
    let actual = serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();
    assert_eq!(
        actual,
        serde_json::json!([{
          "file": "src/Button.vue",
          "messages": [{
            "ruleId": "css/no-hardcoded-values",
            "ruleDocsPath": "docs/content/rules/musea-and-css.md",
            "severity": 2,
            "message": "[vize:css/no-hardcoded-values] Consider using a CSS variable for this color value",
            "line": 1,
            "column": 15,
            "endLine": 2,
            "endColumn": 10,
            "help": "Use var(--color-name) for consistent theming"
          }, {
            "ruleId": "css/no-important",
            "ruleDocsPath": "docs/content/rules/musea-and-css.md",
            "severity": 2,
            "message": "[vize:css/no-important] Avoid using !important as it makes styles harder to override",
            "line": 2,
            "column": 22,
            "endLine": 2,
            "endColumn": 32,
            "help": "Use more specific selectors or reorganize CSS specificity instead"
          }],
          "errorCount": 2,
          "warningCount": 0
        }]),
        "{}",
        output_details(&output),
    );
}
