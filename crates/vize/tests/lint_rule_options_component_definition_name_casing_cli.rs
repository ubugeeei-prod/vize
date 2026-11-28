#![cfg(test)]

use crate::{
    lint_rule_options_casing_cli::{output_details, temp_project_dir, write_project_file},
    support::vite_config,
};
use std::{fs, process::Command};
use vize_l0::String;

const RULE: &str = "vue/component-definition-name-casing";

const FILES: [(&str, &str); 5] = [
    (
        "DefinitionCasing.vue",
        "<script lang=\"ts\">\nimport { defineComponent } from \"vue\";\nexport default defineComponent({\n  name: \"definition_casing\",\n});\n</script>\n<template><div /></template>\n",
    ),
    (
        "DefinitionCasingSetup.vue",
        "<script setup lang=\"ts\">\ndefineOptions({ name: \"definition_casing_setup\" });\n</script>\n<template><div /></template>\n",
    ),
    (
        "DefinitionCasingApp.vue",
        "<script setup lang=\"ts\">\nimport { createApp } from \"vue\";\nimport HelperChild from \"./HelperChild.vue\";\nconst app = createApp({});\napp.component(\"helper_child\", HelperChild);\n</script>\n<template><div /></template>\n",
    ),
    ("HelperChild.vue", "<template><div /></template>\n"),
    (
        "PascalName.vue",
        "<script setup lang=\"ts\">\ndefineOptions({ name: \"PascalName\" });\n</script>\n<template><div /></template>\n",
    ),
];

/// `(file, line, column, message)` of every report of the rule.
fn reports(rule_options: &str) -> Vec<(String, u64, u64, String)> {
    let project_root = temp_project_dir();
    for (path, source) in FILES {
        write_project_file(&project_root, path, source);
    }
    write_project_file(
        &project_root,
        "vite.config.mjs",
        &vite_config::source(
            String::from_iter([
                r#"{ "linter": { "rules": { "vue/component-definition-name-casing": "error" }"#,
                rule_options,
                " } }",
            ])
            .as_bytes(),
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
            "./*.vue",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{}", output_details(&output));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let mut found = Vec::new();
    for file in report.as_array().unwrap() {
        for message in file["messages"].as_array().unwrap() {
            if message["ruleId"] != RULE {
                continue;
            }
            assert_eq!(message["severity"], 2, "{}", output_details(&output));
            found.push((
                String::from(file["file"].as_str().unwrap().trim_start_matches("./")),
                message["line"].as_u64().unwrap(),
                message["column"].as_u64().unwrap(),
                String::from(message["message"].as_str().unwrap()),
            ));
        }
    }
    found.sort();
    let _ = fs::remove_dir_all(project_root);
    found
}

fn expected(rows: &[(&str, u64, u64, &str)]) -> Vec<(String, u64, u64, String)> {
    rows.iter()
        .map(|&(file, line, column, message)| {
            (
                String::from(file),
                line,
                column,
                String::from_iter(["[vize:", RULE, "] ", message]),
            )
        })
        .collect()
}

#[test]
fn every_definition_name_source_is_checked_in_pascal_case_by_default() {
    let pascal = expected(&[
        (
            "DefinitionCasing.vue",
            4,
            9,
            "Component name 'definition_casing' is not PascalCase",
        ),
        (
            "DefinitionCasingApp.vue",
            5,
            15,
            "Component name 'helper_child' is not PascalCase",
        ),
        (
            "DefinitionCasingSetup.vue",
            2,
            23,
            "Component name 'definition_casing_setup' is not PascalCase",
        ),
    ]);
    assert_eq!(reports(""), pascal);
    assert_eq!(
        reports(r#", "ruleOptions": { "vue/component-definition-name-casing": "PascalCase" }"#),
        pascal
    );
}

#[test]
fn the_kebab_case_option_reports_pascal_case_names_instead() {
    assert_eq!(
        reports(r#", "ruleOptions": { "vue/component-definition-name-casing": "kebab-case" }"#),
        expected(&[
            (
                "DefinitionCasing.vue",
                4,
                9,
                "Component name 'definition_casing' is not kebab-case",
            ),
            (
                "DefinitionCasingApp.vue",
                5,
                15,
                "Component name 'helper_child' is not kebab-case",
            ),
            (
                "DefinitionCasingSetup.vue",
                2,
                23,
                "Component name 'definition_casing_setup' is not kebab-case",
            ),
            (
                "PascalName.vue",
                2,
                23,
                "Component name 'PascalName' is not kebab-case",
            ),
        ])
    );
}
