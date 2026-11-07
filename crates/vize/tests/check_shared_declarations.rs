//! A project-references solution reports the same diagnostics whether an
//! importer mirrors the referenced project's sources or its emitted
//! declarations (#12102), and each planted error is reported exactly once by
//! its owning program (#11642).
#![expect(clippy::disallowed_types, reason = "fixtures and JSON use std strings")]

use crate::support::corsa_requirement;
use serde_json::Value;
use std::{path::Path, process::Command};

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn resolve_test_corsa_path() -> Option<String> {
    let workspace_bin = workspace_root().join("node_modules/.bin/tsgo");
    workspace_bin
        .exists()
        .then(|| workspace_bin.to_string_lossy().into_owned())
}

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

/// `kit` publishes a button with a `count: number` prop and a global
/// registration; `app` references `kit`, uses the global and the import, and
/// passes a string to `count`. `kit` itself misuses its own prop once.
fn create_solution(name: &str) -> std::path::PathBuf {
    let root = workspace_root()
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(format!("{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(
        workspace_root().join("node_modules"),
        root.join("node_modules"),
    )
    .unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(
        workspace_root().join("node_modules"),
        root.join("node_modules"),
    )
    .unwrap();
    write(
        &root,
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"./packages/kit"},{"path":"./packages/app"}]}"#,
    );
    write(
        &root,
        "tsconfig.base.json",
        r#"{"compilerOptions":{"strict":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","composite":true,"emitDeclarationOnly":true,"skipLibCheck":true}}"#,
    );
    write(
        &root,
        "packages/kit/tsconfig.json",
        r#"{"extends":"../../tsconfig.base.json","compilerOptions":{"rootDir":"src","outDir":".out"},"include":["src/**/*"]}"#,
    );
    write(
        &root,
        "packages/app/tsconfig.json",
        r#"{"extends":"../../tsconfig.base.json","compilerOptions":{"rootDir":"src","outDir":".out"},"include":["src/**/*"],"references":[{"path":"../kit"}]}"#,
    );
    write(
        &root,
        "packages/kit/src/KitButton.vue",
        "<script setup lang=\"ts\">\ndefineProps<{ count: number }>();\n</script>\n<template><button>{{ count }}</button></template>\n",
    );
    write(
        &root,
        "packages/kit/src/KitPanel.vue",
        "<script setup lang=\"ts\">\nimport KitButton from './KitButton.vue';\n</script>\n<template><KitButton :count=\"'kit-wrong'\" /></template>\n",
    );
    write(
        &root,
        "packages/kit/src/components.d.ts",
        "import 'vue';\ndeclare module 'vue' {\n  export interface GlobalComponents {\n    KitButton: typeof import('./KitButton.vue')['default'];\n  }\n}\n",
    );
    write(
        &root,
        "packages/app/src/App.vue",
        "<script setup lang=\"ts\">\nimport KitPanel from '../../kit/src/KitPanel.vue';\nimport KitButton from '../../kit/src/KitButton.vue';\n</script>\n<template><KitPanel /><KitButton :count=\"'app-wrong'\" /><KitButton :count=\"1\" /></template>\n",
    );
    root
}

fn check(root: &Path, corsa: &str, share: bool) -> Vec<(String, u64, String)> {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .env("CORSA_PATH", corsa)
        .env(
            "VIZE_CHECK_SHARE_DECLARATIONS",
            if share { "1" } else { "0" },
        )
        .args([
            "check",
            "--quiet",
            "--format",
            "json",
            "--tsconfig",
            "tsconfig.json",
        ])
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    let report: Value =
        serde_json::from_str(stdout).unwrap_or_else(|error| panic!("{error}: {stdout}\n{stderr}"));
    let mut diagnostics: Vec<(String, u64, String)> = report["files"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|file| {
            let path = file["file"].as_str().unwrap().replace('\\', "/");
            file["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .map(move |diagnostic| {
                    // `error:5:34 [TS2322] message`
                    let text = diagnostic.as_str().unwrap();
                    let (head, _) = text.split_once("] ").unwrap();
                    let (position, code) = head.split_once(" [").unwrap();
                    let line = position.split(':').nth(1).unwrap().parse::<u64>().unwrap();
                    (
                        path.rsplit("packages/").next().unwrap().to_owned(),
                        line,
                        code.to_owned(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect();
    diagnostics.sort();
    diagnostics
}

#[test]
fn shared_declarations_report_the_same_errors_once_per_owner() {
    let Some(corsa) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let root = create_solution("check-shared-declarations");
    let shared = check(&root, &corsa, true);
    let sources = check(&root, &corsa, false);
    assert_eq!(
        shared,
        vec![
            ("app/src/App.vue".to_owned(), 5, "TS2322".to_owned()),
            ("kit/src/KitPanel.vue".to_owned(), 4, "TS2322".to_owned()),
        ],
        "{shared:?}"
    );
    assert_eq!(shared, sources);
    let _ = std::fs::remove_dir_all(&root);
}
