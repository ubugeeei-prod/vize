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

#[test]
fn configured_script_rules_reach_standalone_html_inline_scripts() {
    let project_root = tempfile::tempdir().unwrap();
    write_file(
        project_root.path(),
        "vize.config.json",
        r#"{
  "linter": {
    "preset": "incremental",
    "rules": {
      "script/no-next-tick": "error"
    }
  }
}"#,
    );
    write_file(
        project_root.path(),
        "index.html",
        r#"<!doctype html>
<html>
<body>
  <div id="app"></div>
  <script type="module">
import { nextTick } from "vue"
await nextTick()
  </script>
</body>
</html>
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
            "index.html",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1), "{}", output_details(&output));
    let actual = serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();
    assert_eq!(
        actual,
        serde_json::json!([{
          "file": "index.html",
          "messages": [{
            "ruleId": "script/no-next-tick",
            "ruleDocsPath": "docs/content/rules/type-and-script.md",
            "severity": 2,
            "message": "[vize:script/no-next-tick] nextTick import is not supported in Vapor-oriented components",
            "line": 6,
            "column": 10,
            "endLine": 6,
            "endColumn": 18,
            "help": "Remove nextTick() usage and rely on explicit lifecycle boundaries like onMounted() or direct reactive flow instead."
          }, {
            "ruleId": "script/no-next-tick",
            "ruleDocsPath": "docs/content/rules/type-and-script.md",
            "severity": 2,
            "message": "[vize:script/no-next-tick] nextTick() is not supported in Vapor-oriented components",
            "line": 7,
            "column": 7,
            "endLine": 7,
            "endColumn": 15,
            "help": "Avoid post-render scheduling with nextTick(). Prefer onMounted(), template refs, or a control flow that does not depend on a DOM flush boundary."
          }],
          "errorCount": 2,
          "warningCount": 0
        }]),
        "{}",
        output_details(&output),
    );
}
