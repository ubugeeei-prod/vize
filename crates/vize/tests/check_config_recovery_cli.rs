//! A failed optional project-config evaluation must explain the explicit bypass.
#![expect(clippy::disallowed_types, reason = "complete CLI process packets")]

use crate::support::{config_refusal, corsa_path, corsa_requirement, vue_stub};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

const HINT: &str =
    "Hint: use `vize check --no-config` to type-check without loading project configuration.\n";
const CONFIG: &str = "export default async () => { await Promise.resolve(); throw new TypeError('owned-check-config-recovery'); };\n";
const CLEAN: &str = "<script setup lang=\"ts\">\nconst n: number = 1;\n</script>\n<template><div>{{ n }}</div></template>\n";
const BROKEN: &str = "<script setup lang=\"ts\">\nconst n: number = 'wrong';\n</script>\n<template><div>{{ n }}</div></template>\n";

#[test]
fn discovered_and_explicit_config_failures_preserve_the_error_and_explain_no_config() {
    for name in ["vite.config.mjs", "vite.config.ts"] {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        fs::write(root.join(name), CONFIG).unwrap();
        fs::write(root.join("A.vue"), CLEAN).unwrap();
        for (extra, selected) in [
            (Vec::new(), root.join(name)),
            (vec!["--config", name], name.into()),
        ] {
            let output = Command::new(env!("CARGO_BIN_EXE_vize"))
                .current_dir(&root)
                .env("NODE_OPTIONS", config_refusal::NODE_OPTIONS)
                .args(["check", "--format", "json"])
                .args(extra)
                .output()
                .unwrap();
            config_refusal::assert_error_with_suffix(
                &output,
                &selected,
                &root,
                "TypeError: owned-check-config-recovery",
                2,
                HINT,
            );
            assert_eq!(fs::read_to_string(root.join(name)).unwrap(), CONFIG);
            assert_eq!(fs::read_to_string(root.join("A.vue")).unwrap(), CLEAN);
        }
    }
}

#[test]
fn no_config_bypasses_failed_evaluation_and_still_checks_authored_type_errors() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let Some(corsa) = corsa_requirement::required_or_skip(corsa_path::resolve(workspace)) else {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::write(root.join("vite.config.ts"), CONFIG).unwrap();
    fs::write(root.join("tsconfig.json"), r#"{
  "compilerOptions": { "strict": true, "target": "ES2022", "module": "ESNext", "moduleResolution": "bundler", "noEmit": true },
  "include": ["A.vue"]
}"#).unwrap();
    vue_stub::install_vue_jsx_type_stub(root);
    for (source, status, diagnostics) in [
        (CLEAN, 0, Vec::new()),
        (
            BROKEN,
            1,
            vec!["error:2:7 [TS2322] Type 'string' is not assignable to type 'number'."],
        ),
    ] {
        fs::write(root.join("A.vue"), source).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(root)
            .env("CORSA_PATH", &corsa)
            .args(["check", "--no-config", "--format", "json"])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(status), "{output:?}");
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            report["files"],
            json!([{"file":"A.vue","diagnostics":diagnostics}])
        );
        assert_eq!(report["fileCount"], json!(1));
        assert_eq!(report["errorCount"], json!(status));
        assert_eq!(report["warningCount"], json!(0));
        assert_eq!(
            fs::read_to_string(root.join("vite.config.ts")).unwrap(),
            CONFIG
        );
        assert_eq!(fs::read_to_string(root.join("A.vue")).unwrap(), source);
    }
}
