#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;
#[path = "support/nuxt_cli.rs"]
mod nuxt_cli;

use std::{path::Path, process::Command};

fn write(root: &Path, name: &str, source: &str) {
    let path = root.join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, source).unwrap();
}

#[test]
fn configured_pattern_check_distinguishes_errors_warnings_and_flag_off() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(nuxt_cli::resolve_test_corsa_path())
    else {
        return;
    };
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    write(
        root,
        "tsconfig.json",
        r#"{"compilerOptions":{"strict":true,"target":"ESNext","module":"ESNext","moduleResolution":"bundler","skipLibCheck":true},"include":["src/**/*"]}"#,
    );
    write(
        root,
        "vize.config.json",
        r#"{"experimentals":{"patternedTemplate":true}}"#,
    );
    write(
        root,
        "node_modules/vue/package.json",
        r#"{"name":"vue","types":"index.d.ts"}"#,
    );
    write(
        root,
        "node_modules/vue/index.d.ts",
        "export interface Ref<T = unknown> { value: T }",
    );
    let check = || {
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(root)
            .env("CORSA_PATH", &corsa_path)
            .args(["check", "src/App.vue", "--format", "json"])
            .output()
            .unwrap();
        let result: serde_json::Value =
            serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
                panic!("{error}: {}", std::str::from_utf8(&output.stderr).unwrap())
            });
        (output.status.success(), result)
    };
    let source = r#"<script setup lang="ts">const state = 'a' as 'a' | 'b';</script>
<template v-match="state"><p v-when="'a'"/><p v-when="'a'"/><p v-when="'b'"/></template>"#;
    write(root, "src/App.vue", source);
    let (success, result) = check();
    assert!(success, "{result}");
    assert_eq!(result["errorCount"], 0, "{result}");
    assert_eq!(result["warningCount"], 1, "{result}");

    write(
        root,
        "src/App.vue",
        &source.replace("<p v-when=\"'b'\"/>", ""),
    );
    let (success, result) = check();
    assert!(!success, "{result}");
    assert_eq!(result["errorCount"], 1, "{result}");

    write(root, "src/App.vue", source);
    write(
        root,
        "vize.config.json",
        r#"{"experimentals":{"patternedTemplate":false}}"#,
    );
    let (success, result) = check();
    assert!(!success, "{result}");
    assert!(result["errorCount"].as_u64().unwrap() > 0, "{result}");
}
