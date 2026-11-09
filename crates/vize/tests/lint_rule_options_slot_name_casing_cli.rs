#![cfg(test)]

use crate::{
    lint_rule_options_casing_cli::{output_details, temp_project_dir, write_project_file},
    support::vite_config,
};
use std::{fs, process::Command};

#[test]
fn slot_name_casing_accepts_the_upstream_direct_style_option() {
    let project_root = temp_project_dir();
    write_project_file(
        &project_root,
        "src/Panel.vue",
        r#"<template><slot name="row-actions" /><slot name="rowActions" /></template>"#,
    );
    write_project_file(
        &project_root,
        "vite.config.mjs",
        &vite_config::source(
            r#"{
  "linter": {
    "preset": "incremental",
    "rules": { "vue/slot-name-casing": "error" },
    "ruleOptions": { "vue/slot-name-casing": "kebab-case" }
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
            "src/Panel.vue",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{}", output_details(&output));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let messages = report[0]["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 1, "{}", output_details(&output));
    assert_eq!(messages[0]["ruleId"], "vue/slot-name-casing");
    assert_eq!(
        messages[0]["message"],
        "[vize:vue/slot-name-casing] Slot name 'rowActions' is not kebab-case"
    );

    let _ = fs::remove_dir_all(project_root);
}
