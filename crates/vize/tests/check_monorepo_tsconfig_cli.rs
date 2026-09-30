#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use std::{path::Path, process::Command};

#[test]
fn root_check_uses_each_package_tsconfig_and_solution_reference() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(
        std::env::var_os("CORSA_PATH").map(|path| path.to_string_lossy().into_owned()),
    ) else {
        return;
    };
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "package.json", r#"{"private":true}"#);
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"moduleResolution":"bundler"},"include":["*.ts"]}"#,
    );
    for (package, solution) in [("web", false), ("admin", true)] {
        write(
            root.path(),
            &format!("packages/{package}/package.json"),
            if solution {
                r##"{"private":true,"type":"module","imports":{"#lib/*":"./src/lib/*/index.ts"}}"##
            } else {
                r#"{"private":true,"type":"module"}"#
            },
        );
        let config = if solution {
            r#"{"compilerOptions":{"strict":true,"module":"preserve","moduleResolution":"bundler","noEmit":true},"include":["src/**/*"]}"#
        } else {
            r##"{"compilerOptions":{"strict":true,"module":"preserve","moduleResolution":"bundler","noEmit":true,"paths":{"#lib/*":["./src/lib/*"]}},"include":["src/**/*"]}"##
        };
        if solution {
            write(
                root.path(),
                &format!("packages/{package}/tsconfig.json"),
                r#"{"files":[],"references":[{"path":"./tsconfig.app.json"}]}"#,
            );
            write(
                root.path(),
                &format!("packages/{package}/tsconfig.app.json"),
                config,
            );
        } else {
            write(
                root.path(),
                &format!("packages/{package}/tsconfig.json"),
                config,
            );
        }
        write(
            root.path(),
            &format!("packages/{package}/src/lib/greet/index.ts"),
            "export const greet = (name: string): string => `hi ${name}`;",
        );
        write(
            root.path(),
            &format!("packages/{package}/src/App.vue"),
            "<script setup lang=\"ts\">import { greet } from '#lib/greet'; const message: string = greet('vize')</script><template>{{ message }}</template>",
        );
    }
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root.path())
        .env("CORSA_PATH", corsa_path)
        .args(["check", "packages", "--format", "json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(report["errorCount"], 0, "{report:#}");
    assert_eq!(report["files"].as_array().unwrap().len(), 2, "{report:#}");
}

#[test]
fn explicit_javascript_uses_its_package_config_even_when_root_denies_allow_js() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(
        std::env::var_os("CORSA_PATH").map(|path| path.to_string_lossy().into_owned()),
    ) else {
        return;
    };
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"allowJs":false}}"#,
    );
    write(
        root.path(),
        "packages/app/tsconfig.json",
        r#"{"compilerOptions":{"allowJs":true,"checkJs":true,"noEmit":true},"include":["src/**/*"]}"#,
    );
    write(
        root.path(),
        "packages/app/src/value.js",
        "export const value = 1;\n",
    );
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root.path())
        .env("CORSA_PATH", corsa_path)
        .args(["check", "packages/app/src/value.js", "--format", "json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(output.status.success(), "{stdout}");
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(report["fileCount"], 1, "{report:#}");
    assert_eq!(
        report["programs"][0]["tsconfig"],
        "packages/app/tsconfig.json"
    );
}

fn write(root: &Path, path: &str, content: &str) {
    let path = root.join(path);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}
