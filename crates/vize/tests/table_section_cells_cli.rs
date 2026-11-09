#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "subprocess fixture output")]

use std::{fs, process::Command};

const APP: &str = r#"<template>
  <table>
    <thead>
      <th>Name</th>
      <th>Grade</th>
    </thead>
  </table>
</template>
"#;

const EXPECTED_BUILD: &str = r#"import { createElementVNode as _createElementVNode, openBlock as _openBlock, createElementBlock as _createElementBlock } from "vue"

export function render(_ctx, _cache) {
  return (_openBlock(), _createElementBlock("table", null, [
    _createElementVNode("thead", null, [
      _createElementVNode("th", null, "Name"),
      _createElementVNode("th", null, "Grade")
    ])
  ]))
}"#;

const EXPECTED_LINT_REPORT: &str = r#"[
  {
    "file": "App.vue",
    "messages": [],
    "errorCount": 0,
    "warningCount": 0
  }
]"#;

#[test]
fn build_and_lint_preserve_authored_table_section_cells() {
    let project = tempfile::tempdir().expect("temporary project");
    fs::write(project.path().join("App.vue"), APP).expect("write SFC");

    let build = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args(["build", "--format", "js", "App.vue", "--output", "dist"])
        .output()
        .expect("run build");
    assert!(
        build.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );
    let built = fs::read_to_string(project.path().join("dist/App.js")).expect("built module");
    assert_eq!(built, EXPECTED_BUILD);

    let config = project.path().join("serialized-config.json");
    fs::write(
        &config,
        r#"{
  "linter": {
    "preset": "incremental",
    "rules": { "vue/permitted-contents": "off" }
  }
}"#,
    )
    .expect("write config");

    let lint = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args(["lint", "--config"])
        .arg(&config)
        .args(["--format", "json", "App.vue"])
        .output()
        .expect("run lint");
    assert!(
        lint.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&lint.stdout),
        String::from_utf8_lossy(&lint.stderr)
    );
    let report = String::from_utf8(lint.stdout).expect("UTF-8 lint report");
    assert_eq!(report, EXPECTED_LINT_REPORT);
}
