#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use crate::support::vite_config;
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
        "vite.config.mjs",
        &vite_config::source(
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
        ),
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
            "vite.config.mjs",
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

#[test]
fn opinionated_lint_allows_generated_design_token_js_data_object() {
    let project_root = tempfile::tempdir().unwrap();
    write_project_file(
        project_root.path(),
        "src/tokens.js",
        r##"/**
 * Do not edit directly, this file was auto-generated.
 */
export default {
  color: {
    white: { value: "#ffffff" },
    primary: { value: "#4bc4cc" },
  },
};
"##,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root.path())
        .args([
            "lint",
            "--preset",
            "opinionated",
            "--no-config",
            "--format",
            "json",
            "src",
        ])
        .output()
        .unwrap();

    assert!(output.status.success(), "{}", output_details(&output));
    assert_eq!(
        json_rule_ids(&output),
        BTreeSet::new(),
        "{}",
        output_details(&output)
    );
}

#[test]
fn binding_values_keep_same_name_shorthand_and_reject_authored_empty_values() {
    let project_root = tempfile::tempdir().unwrap();
    write_project_file(
        project_root.path(),
        "vite.config.mjs",
        r#"export default { vue: { linter: { preset: "incremental", rules: { "vue/valid-v-bind": "error" } } } };"#,
    );
    for (attribute, invalid) in [
        (r#":label="""#, true),
        (r#"v-bind:label="  ""#, true),
        (r#".label="""#, true),
        (r#":[name]="""#, true),
        (r#"v-bind="""#, true),
        (":label", false),
        ("v-bind:label", false),
        (r#":label="label""#, false),
        (r#":[name]="label""#, false),
        (r#"v-bind="{title: 'hello'}""#, false),
    ] {
        let source = format!("<template>\n  <!-- あ😀 -->\n  <div {attribute} />\n</template>\n");
        write_project_file(project_root.path(), "App.vue", &source);
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(project_root.path())
            .args([
                "lint",
                "--config",
                "vite.config.mjs",
                "--format",
                "json",
                "--help-level",
                "none",
                "App.vue",
            ])
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(i32::from(invalid)),
            "{attribute}: {}",
            output_details(&output)
        );
        assert_eq!(output.stderr, Vec::<u8>::new(), "{attribute}");
        let messages = if invalid {
            serde_json::json!([{
                "ruleId": "vue/valid-v-bind",
                "ruleDocsPath": "docs/content/rules/vue.md",
                "severity": 2,
                "message": "[vize:vue/valid-v-bind] v-bind requires an expression",
                "line": 3,
                "column": 8,
                "endLine": 3,
                "endColumn": 8 + attribute.len()
            }])
        } else {
            serde_json::json!([])
        };
        let expected = serde_json::json!([{
            "file": "App.vue",
            "messages": messages,
            "errorCount": usize::from(invalid),
            "warningCount": 0
        }]);
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap(),
            expected,
            "{attribute}: {}",
            output_details(&output)
        );
        assert_eq!(
            fs::read(project_root.path().join("App.vue")).unwrap(),
            source.as_bytes()
        );
    }
}
