//! Nuxt's fallback alias wrapper must not turn a TS 5-era project config into
//! TypeScript 7 option errors (#3682).

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn resolve_test_corsa_path() -> Option<PathBuf> {
    let root = workspace_root();
    [
        root.parent()?.join("corsa-bind/.cache/tsgo"),
        root.join("node_modules/.bin/tsgo"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}

fn project(name: &str, script: &str) -> PathBuf {
    static NEXT_CASE_ID: AtomicUsize = AtomicUsize::new(0);
    let case_id = NEXT_CASE_ID.fetch_add(1, Ordering::Relaxed);
    let root = workspace_root()
        .join("target/vize-tests/tests")
        .join(format!(
            "nuxt-legacy-tsconfig-{name}-{}-{case_id}",
            std::process::id()
        ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).unwrap();
    write(&root, "nuxt.config.ts", "export default {};\n");
    write(
        &root,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Node",
    "baseUrl": ".",
    "paths": {
      "~/*": ["./*"],
      "@/*": ["./*"]
    },
    "noEmit": true,
    "skipLibCheck": true,
    "types": []
  },
  "include": ["src"]
}
"#,
    );
    write(
        &root,
        "src/App.vue",
        &format!(
            r#"<script setup lang="ts">
{script}
</script>

<template><p>Nuxt</p></template>
"#
        ),
    );
    root
}

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn check(root: &Path, corsa_path: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "--no-config",
            "--tsconfig",
            "tsconfig.json",
            "--no-check-props",
            "--no-check-emits",
            "--no-check-template-bindings",
            "--format",
            "json",
            "src",
        ])
        .output()
        .unwrap()
}

fn output_text(output: &Output) -> (String, String) {
    (
        String::from_utf8(output.stdout.clone()).unwrap(),
        String::from_utf8(output.stderr.clone()).unwrap(),
    )
}

#[test]
fn check_accepts_a_nuxt_fallback_over_a_ts5_era_config() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let root = project("clean", "const count: number = 1;\nvoid count;");
    let output = check(&root, &corsa_path);
    let (stdout, stderr) = output_text(&output);

    assert!(
        output.status.success(),
        "Nuxt fallback check failed:\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["errorCount"], 0, "{stdout}\n{stderr}");
    assert!(!stdout.contains("TS5102") && !stdout.contains("TS5108"));
    assert!(!stdout.contains("tsconfig.nuxt-fallback.json"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn check_still_reports_source_errors_with_the_nuxt_fallback() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let root = project(
        "source-error",
        "const count: number = \"wrong\";\nvoid count;",
    );
    let output = check(&root, &corsa_path);
    let (stdout, stderr) = output_text(&output);

    assert!(!output.status.success(), "{stdout}\n{stderr}");
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["errorCount"], 1, "{stdout}\n{stderr}");
    assert_eq!(json["files"].as_array().unwrap().len(), 1, "{stdout}");
    assert!(
        json["files"][0]["diagnostics"][0]
            .as_str()
            .is_some_and(|diagnostic| diagnostic.contains("[TS2322]")),
        "{stdout}\n{stderr}"
    );
    assert!(!stdout.contains("TS5102") && !stdout.contains("TS5108"));
    assert!(!stdout.contains("tsconfig.nuxt-fallback.json"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn check_still_reports_unrelated_config_errors_with_the_nuxt_fallback() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let root = project("config-error", "const count: number = 1;\nvoid count;");
    let tsconfig = root.join("tsconfig.json");
    let content = std::fs::read_to_string(&tsconfig).unwrap().replace(
        "\"types\": []",
        "\"types\": [],\n    \"nosuchoption\": true",
    );
    std::fs::write(tsconfig, content).unwrap();
    let output = check(&root, &corsa_path);
    let (stdout, stderr) = output_text(&output);

    assert!(!output.status.success(), "{stdout}\n{stderr}");
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["errorCount"], 1, "{stdout}\n{stderr}");
    assert!(stdout.contains("[TS5023]"), "{stdout}\n{stderr}");
    assert!(!stdout.contains("TS5102") && !stdout.contains("TS5108"));

    let _ = std::fs::remove_dir_all(&root);
}
