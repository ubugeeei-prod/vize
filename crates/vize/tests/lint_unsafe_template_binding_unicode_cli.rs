//! Source-built CLI acceptance for the Unicode comment binding regression (#10898).
#![cfg(test)]

use crate::support::{corsa_path, corsa_requirement};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

fn workspace() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
}

fn write(root: &Path, name: &str, contents: &str) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

#[test]
fn unicode_comments_before_arrows_stay_safe_through_the_public_cli() {
    let root = workspace();
    let Some(runtime) = corsa_requirement::required_or_skip(corsa_path::resolve(root)) else {
        return;
    };
    let runtime = fs::canonicalize(runtime).unwrap();
    let project = tempfile::tempdir().unwrap();
    let files = [
        (
            "package.json",
            r#"{"name":"unicode-comment-binding","private":true,"type":"module"}"#,
        ),
        (
            "vite.config.mjs",
            r#"export default { vue: { linter: { preset: "incremental", typeAware: true, rules: { "type/no-unsafe-template-binding": "warn" } } } };"#,
        ),
        (
            "tsconfig.json",
            r#"{
  "compilerOptions": {
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "strict": true,
    "skipLibCheck": true,
    "noEmit": true
  },
  "include": ["src/**/*.vue", "src/**/*.ts"]
}"#,
        ),
        (
            "src/MyInput.vue",
            r#"<script setup lang="ts">
const { rules = [] } = defineProps<{ rules?: (true | string)[] }>();
</script>

<template>
  <input :data-count="rules.length" />
</template>
"#,
        ),
        (
            "src/MyForm.vue",
            r#"<script setup lang="ts">
import MyInput from "./MyInput.vue";

const ids: number[] = [1];
const unsafeRules: any = [];
function check(all: number[], current: number[]): true | string {
  return all.length > 0 || current.length > 0 || "empty";
}
</script>

<template>
  <!-- 1: a line comment with non-ASCII text before an arrow function -->
  <MyInput
    :rules="[
      // éééééééééééééééééééééééééééééé
      check(ids, ids.map(id => id + 1))
    ]"
  />
  <!-- 2: the same comment in ASCII -->
  <MyInput
    :rules="[
      // eeeeeeeeeeeeeeeeeeeeeeeeeeeeee
      check(ids, ids.map(id => id + 1))
    ]"
  />
  <!-- 3: the non-ASCII comment as a block comment -->
  <MyInput
    :rules="[
      /* éééééééééééééééééééééééééééééé */
      check(ids, ids.map(id => id + 1))
    ]"
  />
  <!-- 4: the non-ASCII comment without an arrow function -->
  <MyInput
    :rules="[
      // éééééééééééééééééééééééééééééé
      check(ids, ids.map(String).map(Number))
    ]"
  />
  <!-- Positive control: the rule must still execute and reject authored any. -->
  <MyInput :rules="unsafeRules" />
</template>
"#,
        ),
    ];
    for (name, contents) in files {
        write(project.path(), name, contents);
    }
    crate::support::indexed_access::symlink_vue(root, project.path());

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .env("CORSA_PATH", runtime)
        .env("NO_COLOR", "1")
        .args([
            "lint",
            "-f",
            "json",
            "--help-level",
            "none",
            "-c",
            "vite.config.mjs",
            "src/**/*.vue",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0), "stderr: {:?}", output.stderr);
    assert_eq!(output.stderr, b"");
    let actual: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        actual,
        json!([
            {
                "file":"src/MyForm.vue",
                "messages":[{
                    "ruleId":"type/no-unsafe-template-binding",
                    "ruleDocsPath":"docs/content/rules/type-and-script.md",
                    "severity":1,
                    "message":"[vize:type/no-unsafe-template-binding] Template binding resolves to an unsafe `any` or `unknown` type",
                    "line":41,
                    "column":20,
                    "endLine":41,
                    "endColumn":31
                }],
                "errorCount":0,
                "warningCount":1
            },
            {"file":"src/MyInput.vue","messages":[],"errorCount":0,"warningCount":0}
        ])
    );
    for (name, contents) in files {
        assert_eq!(
            fs::read(project.path().join(name)).unwrap(),
            contents.as_bytes(),
            "{name} authored bytes"
        );
    }
}
