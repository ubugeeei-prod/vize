#![cfg(test)]

use crate::{
    lint_rule_options_casing_cli::{output_details, temp_project_dir, write_project_file},
    support::vite_config,
};
use std::{fs, process::Command};

#[test]
fn lint_rule_options_accept_real_project_directives_strings_and_components() {
    let project_root = temp_project_dir();
    write_project_file(
        &project_root,
        "src/Allowances.vue",
        r#"<script setup>
const tip = "Copied";
const alt = "Get the app";
const label = "Name";
const name = "";
</script>
<template>
  <button type="button" v-hover-tooltip="tip">{{ tip }}</button>
  <kbd>W</kbd>
  <a href="/retry"><RetryImage src="/retry.png" :alt="alt" /></a>
  <label>{{ label }}<CInputField v-model="name" /></label>
  <button v-unregistered>Translate me</button>
</template>
"#,
    );
    write_project_file(
        &project_root,
        "vite.config.mjs",
        &vite_config::source(
            r#"{
  "linter": {
    "preset": "incremental",
    "rules": {
      "vue/warn-custom-directive": "error",
      "vue/no-bare-strings-in-template": "error",
      "a11y/anchor-has-content": "error",
      "a11y/label-has-for": "error"
    },
    "ruleOptions": {
      "vue/warn-custom-directive": { "ignore": ["hover-tooltip"] },
      "vue/no-bare-strings-in-template": { "allowlist": ["/^[DRWXYmvx]$/u"] },
      "a11y/anchor-has-content": { "accessibleChildren": ["RetryImage"] },
      "a11y/label-has-for": { "controlComponents": ["CInputField"] }
    }
  }
}"#,
        ),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args([
            "lint",
            "--config",
            "vite.config.mjs",
            "--format",
            "json",
            "src/Allowances.vue",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{}", output_details(&output));
    assert_eq!(output.stderr, b"", "{}", output_details(&output));

    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let report: serde_json::Value = serde_json::from_str(stdout).unwrap();
    let rule_ids = report[0]["messages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|message| message["ruleId"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        rule_ids,
        [
            "vue/warn-custom-directive",
            "vue/no-bare-strings-in-template"
        ],
        "{stdout}"
    );

    let _ = fs::remove_dir_all(project_root);
}
