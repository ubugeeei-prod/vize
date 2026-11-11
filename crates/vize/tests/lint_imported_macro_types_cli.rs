//! Imported compiler-macro types feed the same lint facts as inline types.
#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "subprocess fixture diagnostics")]
#![expect(clippy::disallowed_types, reason = "subprocess fixture sources")]

use serde_json::Value;
use std::{collections::BTreeSet, fs, path::Path, process::Command};

fn write(root: &Path, path: &str, source: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, source).unwrap();
}

#[test]
fn imported_define_props_and_emits_types_drive_all_macro_shape_rules() {
    let project = tempfile::tempdir().unwrap();
    write(
        project.path(),
        "lint.json",
        r#"{
  "vue": {
    "linter": {
    "preset": "incremental",
    "rules": {
      "vue/prop-name-casing": "error",
      "vue/no-unused-properties": "error",
      "script/no-dupe-keys": "error",
      "script/no-required-prop-with-default": "error",
      "vue/no-mutating-props": "error",
      "vue/no-undefined-refs": "error",
      "vue/no-template-shadow": "error",
      "script/require-explicit-emits": "error",
      "script/no-unused-emit-declarations": "error"
    }
    }
  }
}"#,
    );
    write(
        project.path(),
        "src/types.ts",
        r#"export interface CardProps {
  my_title: string;
  count: number;
  note?: string;
  extra?: string;
  item?: string;
  list: string[];
}

export interface CardEmits {
  save: [];
  close: [];
}
"#,
    );
    write(
        project.path(),
        "src/ImportedCard.vue",
        r#"<script setup lang="ts">
import { ref } from "vue";
import type { CardEmits, CardProps } from "./types";

const { my_title = "Card" } = defineProps<CardProps>();
const emit = defineEmits<CardEmits>();
const extra = ref(1);
</script>

<template>
  <button type="button" @click="count++">{{ my_title }} {{ extra }} {{ missing }}</button>
  <button type="button" @click="emit('save')">save</button>
  <button type="button" @click="$emit('undeclared')">other</button>
  <ul><li v-for="item in list" :key="item">{{ item }}</li></ul>
</template>
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args([
            "lint",
            "--config",
            "lint.json",
            "--format",
            "json",
            "--help-level",
            "none",
            "src/ImportedCard.vue",
        ])
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert_eq!(output.stderr, b"");

    let reports: Value = serde_json::from_slice(&output.stdout).unwrap();
    let report = &reports.as_array().unwrap()[0];
    assert_eq!(
        report["errorCount"],
        10,
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(report["warningCount"], 0);
    let mut actual: Vec<_> = report["messages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|message| {
            (
                message["ruleId"].as_str().unwrap(),
                message["line"].as_u64().unwrap(),
                message["column"].as_u64().unwrap(),
            )
        })
        .collect();
    actual.sort();
    assert_eq!(
        actual,
        [
            ("script/no-dupe-keys", 7, 7),
            ("script/no-required-prop-with-default", 5, 43),
            ("script/no-unused-emit-declarations", 6, 26),
            ("script/require-explicit-emits", 13, 39),
            ("vue/no-mutating-props", 11, 33),
            ("vue/no-template-shadow", 14, 18),
            ("vue/no-undefined-refs", 11, 72),
            ("vue/no-unused-properties", 5, 43),
            ("vue/no-unused-properties", 5, 43),
            ("vue/prop-name-casing", 5, 43),
        ]
    );

    // Cross-file resolution is script-rule demand, not an accidental side
    // effect of enabling one of the semantic template rules above.
    write(
        project.path(),
        "lint.json",
        r#"{"vue":{"linter":{"preset":"incremental","rules":{
          "script/no-dupe-keys":"error",
          "script/no-required-prop-with-default":"error",
          "script/require-explicit-emits":"error",
          "script/no-unused-emit-declarations":"error"
        }}}}"#,
    );
    let script_only = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args([
            "lint",
            "--config",
            "lint.json",
            "--format",
            "json",
            "--help-level",
            "none",
            "src/ImportedCard.vue",
        ])
        .output()
        .unwrap();
    assert_eq!(script_only.status.code(), Some(1));
    assert_eq!(script_only.stderr, b"");
    let script_only: Value = serde_json::from_slice(&script_only.stdout).unwrap();
    let rules: BTreeSet<_> = script_only[0]["messages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|message| message["ruleId"].as_str().unwrap())
        .collect();
    assert_eq!(
        rules,
        BTreeSet::from([
            "script/no-dupe-keys",
            "script/no-required-prop-with-default",
            "script/no-unused-emit-declarations",
            "script/require-explicit-emits",
        ])
    );
}

#[test]
fn imported_prop_default_rules_request_the_type_world_independently() {
    let project = tempfile::tempdir().unwrap();
    write(
        project.path(),
        "src/types.ts",
        r#"export interface Props {
  enabled?: boolean;
  label?: string;
}
"#,
    );
    write(
        project.path(),
        "src/ImportedDefaults.vue",
        r#"<script setup lang="ts">
import type { Props } from "./types";

withDefaults(defineProps<Props>(), { enabled: true });
</script>
"#,
    );

    for rule in ["script/no-boolean-default", "script/require-default-prop"] {
        write(
            project.path(),
            "lint.json",
            &format!(
                r#"{{"vue":{{"linter":{{"preset":"incremental","rules":{{"{rule}":"error"}}}}}}}}"#,
            ),
        );
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(project.path())
            .args([
                "lint",
                "--config",
                "lint.json",
                "--format",
                "json",
                "--help-level",
                "none",
                "src/ImportedDefaults.vue",
            ])
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(1),
            "{rule}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        assert_eq!(output.stderr, b"");
        let reports: Value = serde_json::from_slice(&output.stdout).unwrap();
        let messages = reports[0]["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 1, "{rule}: {messages:?}");
        assert_eq!(messages[0]["ruleId"], rule);
    }
}
