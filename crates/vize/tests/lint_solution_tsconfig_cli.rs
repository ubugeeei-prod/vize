//! Type-aware lint follows solution-style project ownership (#10176).
#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "subprocess fixture paths")]
#![expect(clippy::disallowed_types, reason = "subprocess fixture strings")]

use crate::support::corsa_path;
use crate::support::corsa_requirement;
use crate::support::indexed_access as support;

use serde_json::Value;
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
            "--help-level",
            "none",
            "--format",
            "json",
            "src/App.vue",
        ])
        .output()
        .unwrap()
}

fn assert_clean(output: &std::process::Output) {
    assert_eq!(output.status.code(), Some(0), "stderr: {:?}", output.stderr);
    assert_eq!(output.stderr, b"");
    let reports: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(reports[0]["errorCount"], 0, "{reports:#}");
    assert_eq!(reports[0]["warningCount"], 0, "{reports:#}");
    assert_eq!(reports[0]["messages"], serde_json::json!([]), "{reports:#}");
}

#[test]
fn aliases_and_ambient_types_follow_references_or_the_configured_project() {
    let root = workspace();
    let runtime: Option<String> = corsa_requirement::required_or_skip(corsa_path::resolve(root));
    let Some(runtime) = runtime else { return };
    let project = tempfile::tempdir().unwrap();
    support::symlink_vue(root, project.path());
    write(
        project.path(),
        "package.json",
        r#"{"name":"solution-type-aware-lint","private":true,"type":"module"}"#,
    );
    write(
        project.path(),
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"./tsconfig.app.json"}]}"#,
    );
    write(
        project.path(),
        "tsconfig.app.json",
        r#"{"compilerOptions":{"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","strict":true,"noEmit":true,"jsx":"preserve","skipLibCheck":true,"types":[],"paths":{"@/*":["./src/*"]}},"include":["env.d.ts","src/**/*.ts","src/**/*.vue"]}"#,
    );
    write(
        project.path(),
        "lint.json",
        r#"{"vue":{"linter":{"preset":"incremental","typeAware":true,"rules":{"type/no-unsafe-template-binding":"warn"}}}}"#,
    );
    write(
        project.path(),
        "env.d.ts",
        "declare const __APP_VERSION__: string;\n",
    );
    write(
        project.path(),
        "src/labels.ts",
        "export const greeting: string = \"hello\";\n",
    );
    write(
        project.path(),
        "src/App.vue",
        r#"<script setup lang="ts">
import { greeting } from "@/labels";
const version = __APP_VERSION__;
</script>
<template>
  <p>{{ greeting.toUpperCase() }}</p>
  <footer>{{ version.toUpperCase() }}</footer>
</template>
"#,
    );

    assert_clean(&lint(project.path(), &runtime));

    write(project.path(), "tsconfig.json", r#"{"files":[]}"#);
    write(
        project.path(),
        "lint.json",
        r#"{"vue":{"typeChecker":{"tsconfig":"tsconfig.app.json"},"linter":{"preset":"incremental","typeAware":true,"rules":{"type/no-unsafe-template-binding":"warn"}}}}"#,
    );
    assert_clean(&lint(project.path(), &runtime));
}
