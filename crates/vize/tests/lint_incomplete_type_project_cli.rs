//! Type-aware lint refuses incomplete TypeScript project configuration (#10005).
#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "subprocess fixture paths")]
#![expect(clippy::disallowed_types, reason = "subprocess fixture strings")]
#[path = "support/corsa_path.rs"]
mod corsa_path;
#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;
#[path = "lint_unchecked_indexed_access_cli/support.rs"]
mod support;

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

fn lint(project: &Path, runtime: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project)
        .env("CORSA_PATH", runtime)
        .args([
            "lint",
            "--config",
            "lint.json",
            "--type-aware",
            "--help-level",
            "none",
            "--format",
            "json",
            "src/**/*.vue",
        ])
        .output()
        .unwrap()
}

#[test]
fn broken_tsconfig_fails_once_without_unsafe_type_rule_flood_and_recovers() {
    let root = workspace();
    let runtime: Option<String> = corsa_requirement::required_or_skip(corsa_path::resolve(root));
    let Some(runtime) = runtime else { return };
    let project = tempfile::tempdir().unwrap();
    support::symlink_vue(root, project.path());
    write(
        project.path(),
        "package.json",
        r#"{"name":"broken-type-project","private":true,"type":"module"}"#,
    );
    write(
        project.path(),
        "tsconfig.json",
        r#"{"extends":"./.generated/tsconfig.base.json","compilerOptions":{"strict":true,"noEmit":true},"include":["src/**/*.vue","lib/**/*.ts"]}"#,
    );
    write(
        project.path(),
        "lint.json",
        r#"{"linter":{"preset":"incremental","typeAware":true,"rules":{"type/no-unsafe-template-binding":"warn"}}}"#,
    );
    write(
        project.path(),
        "lib/data.ts",
        "export interface Row { id: number; name: string }\nexport const rows: Row[] = [{ id: 1, name: \"a\" }];\n",
    );
    for name in ["MyList.vue", "MyOtherList.vue"] {
        write(
            project.path(),
            &format!("src/{name}"),
            r##"<script setup lang="ts">
import { rows } from "#lib/data";
</script>
<template>
  <ul><li v-for="row in rows" :key="row.id">{{ row.name }}</li></ul>
</template>
"##,
        );
    }

    let broken = lint(project.path(), &runtime);
    assert_eq!(broken.status.code(), Some(1));
    assert_eq!(broken.stderr, b"");
    let broken_json: Value = serde_json::from_slice(&broken.stdout).unwrap();
    let reports = broken_json.as_array().unwrap();
    assert_eq!(reports.len(), 2);
    assert_eq!(
        reports
            .iter()
            .map(|report| report["errorCount"].as_u64().unwrap())
            .sum::<u64>(),
        1
    );
    assert_eq!(
        reports
            .iter()
            .map(|report| report["warningCount"].as_u64().unwrap())
            .sum::<u64>(),
        0
    );
    let messages = reports
        .iter()
        .flat_map(|report| report["messages"].as_array().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["ruleId"], "type/project-configuration");
    assert_eq!(messages[0]["severity"], 2);
    assert_eq!(
        messages[0]["message"],
        json!(format!(
            "[vize:type/project-configuration] Cannot load type-aware project configuration {}:\n- File './.generated/tsconfig.base.json' not found.",
            project.path().join("tsconfig.json").display()
        ))
    );
    assert_eq!(project.path().join(".vize/patina").exists(), false);

    write(
        project.path(),
        ".generated/tsconfig.base.json",
        r##"{"compilerOptions":{"module":"ESNext","moduleResolution":"Bundler","target":"ESNext","paths":{"#lib/*":["../lib/*"]}}}"##,
    );
    let repaired = lint(project.path(), &runtime);
    assert_eq!(repaired.status.code(), Some(0));
    assert_eq!(repaired.stderr, b"");
    let repaired_json: Value = serde_json::from_slice(&repaired.stdout).unwrap();
    assert_eq!(
        repaired_json,
        json!([
            {"file":"src/MyList.vue","messages":[],"errorCount":0,"warningCount":0},
            {"file":"src/MyOtherList.vue","messages":[],"errorCount":0,"warningCount":0}
        ])
    );
}
