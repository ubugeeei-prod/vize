#![allow(clippy::disallowed_macros, clippy::disallowed_types)]

use std::{collections::BTreeSet, fs, path::Path, process::Command};

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

fn json_rule_ids(output: &std::process::Output) -> BTreeSet<String> {
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|_| panic!("lint must emit JSON: {}", output_details(output)));
    let files = parsed.as_array().unwrap_or_else(|| {
        panic!(
            "lint JSON root must be an array: {}",
            output_details(output)
        )
    });

    files
        .iter()
        .flat_map(|file| {
            file["messages"]
                .as_array()
                .unwrap_or_else(|| panic!("lint file entry must contain messages: {file:#?}"))
        })
        .map(|message| {
            message["ruleId"]
                .as_str()
                .unwrap_or_else(|| panic!("lint message must contain ruleId: {message:#?}"))
                .to_owned()
        })
        .collect()
}

#[test]
fn configured_rules_reach_every_lint_execution_family() {
    let project_root = tempfile::tempdir().unwrap();
    write_project_file(
        project_root.path(),
        "vize.config.json",
        r#"{
  "linter": {
    "preset": "incremental",
    "rules": {
      "script/no-next-tick": "error",
      "css/no-important": "warn",
      "vue/a11y-img-alt": "warn",
      "a11y/img-alt": "warn",
      "musea/require-title": "error"
    }
  }
}"#,
    );
    write_project_file(
        project_root.path(),
        "src/App.vue",
        r#"<script setup lang="ts">
import { nextTick } from 'vue'
const raw = '<strong>unsafe</strong>'
await nextTick()
</script>

<template>
  <img src="/hero.png">
</template>

<style>
.card { color: red !important; }
</style>
"#,
    );
    write_project_file(
        project_root.path(),
        "src/Button.art.vue",
        r#"<art component="./Button.vue">
  <variant name="default">Default</variant>
</art>
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
            "--max-warnings",
            "0",
            "src/App.vue",
            "src/Button.art.vue",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success(), "{}", output_details(&output));
    assert_eq!(
        json_rule_ids(&output),
        BTreeSet::from([
            "a11y/img-alt".to_owned(),
            "css/no-important".to_owned(),
            "musea/require-title".to_owned(),
            "script/no-next-tick".to_owned(),
            "vue/a11y-img-alt".to_owned(),
        ]),
        "{}",
        output_details(&output)
    );
}
