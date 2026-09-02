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

fn write_css_fixtures(root: &Path) {
    let source = r#"<style>
.a { color: red !important; }
</style>
"#;
    write_file(root, "src/components/Card.vue", source);
    write_file(root, "src/pages/Home.vue", source);
}

fn css_no_important_message(file: &str) -> serde_json::Value {
    serde_json::json!({
      "file": file,
      "messages": [{
        "ruleId": "css/no-important",
        "ruleDocsPath": "docs/content/rules/musea-and-css.md",
        "severity": 2,
        "message": "[vize:css/no-important] Avoid using !important as it makes styles harder to override",
        "line": 2,
        "column": 17,
        "endLine": 2,
        "endColumn": 27,
        "help": "Use more specific selectors or reorganize CSS specificity instead"
      }],
      "errorCount": 1,
      "warningCount": 0
    })
}

fn empty_file_result(file: &str) -> serde_json::Value {
    serde_json::json!({
      "file": file,
      "messages": [],
      "errorCount": 0,
      "warningCount": 0
    })
}

#[test]
fn entry_linter_rules_disable_builtin_css_rules_for_matching_files() {
    let project_root = tempfile::tempdir().unwrap();
    write_file(
        project_root.path(),
        "vize.config.json",
        r#"{
  "linter": {
    "preset": "incremental",
    "rules": { "css/no-important": "error" }
  },
  "entries": [{
    "files": ["src/pages/**/*.vue"],
    "linter": { "rules": { "css/no-important": "off" } }
  }]
}"#,
    );
    write_css_fixtures(project_root.path());

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

    assert_eq!(output.status.code(), Some(1), "{}", output_details(&output));
    let actual = serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();
    assert_eq!(
        actual,
        serde_json::json!([
            css_no_important_message("src/components/Card.vue"),
            empty_file_result("src/pages/Home.vue")
        ]),
        "{}",
        output_details(&output),
    );
}

#[test]
fn entry_linter_rules_enable_builtin_css_rules_for_matching_files() {
    let project_root = tempfile::tempdir().unwrap();
    write_file(
        project_root.path(),
        "vize.config.json",
        r#"{
  "linter": {
    "preset": "incremental",
    "rules": { "css/no-important": "off" }
  },
  "entries": [{
    "files": ["src/pages/**/*.vue"],
    "linter": { "rules": { "css/no-important": "error" } }
  }]
}"#,
    );
    write_css_fixtures(project_root.path());

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

    assert_eq!(output.status.code(), Some(1), "{}", output_details(&output));
    let actual = serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();
    assert_eq!(
        actual,
        serde_json::json!([
            empty_file_result("src/components/Card.vue"),
            css_no_important_message("src/pages/Home.vue")
        ]),
        "{}",
        output_details(&output),
    );
}
