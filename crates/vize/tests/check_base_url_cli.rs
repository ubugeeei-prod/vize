//! `baseUrl` projects check clean end to end (#3886).
//!
//! `vue-tsc` on TypeScript 5.x accepts `baseUrl` silently and resolves bare
//! specifiers relative to it. The native checker removed the option, so before
//! the emulation this exact fixture produced a false `TS2307` on the import, a
//! false `TS5102` on the config, and lost the real `TS2345` behind them.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use std::{path::Path, process::Command};

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn write_file(root: &Path, path: &str, content: &str) {
    let file_path = root.join(path);
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(file_path, content).unwrap();
}

fn resolve_test_corsa_path() -> Option<String> {
    let workspace_root = workspace_root();
    let sibling_cache = workspace_root.parent()?.join("corsa-bind/.cache/tsgo");
    if sibling_cache.exists() {
        return Some(sibling_cache.display().to_string());
    }

    let workspace_bin = workspace_root.join("node_modules/.bin/tsgo");
    workspace_bin
        .exists()
        .then(|| workspace_bin.display().to_string())
}

fn write_base_url_project(project_root: &Path, typescript_version: &str) {
    write_file(
        project_root,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "baseUrl": ".",
    "strict": true,
    "noEmit": true,
    "skipLibCheck": true
  },
  "include": ["src/**/*.ts"]
}"#,
    );
    // The version the narrowing consults: `vue-tsc` resolves its `typescript`
    // peer from here, so this is the toolchain whose verdicts count.
    write_file(
        project_root,
        "node_modules/typescript/package.json",
        &format!(r#"{{ "name": "typescript", "version": "{typescript_version}" }}"#),
    );
    write_file(
        project_root,
        "src/base/greet.ts",
        "export function greet(name: string): string {\n  return `hello ${name}`;\n}\n",
    );
    write_file(
        project_root,
        "src/main.ts",
        r#"import { greet } from "src/base/greet";

export const wrong = greet(42);
"#,
    );
}

#[test]
fn base_url_bare_specifiers_resolve_and_the_config_is_accepted_on_typescript_5() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };

    let project_root = workspace_root()
        .join("target/vize-tests/tests")
        .join(format!("base-url-ts5-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&project_root);
    write_base_url_project(&project_root, "5.8.3");

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", "src", "--no-config", "--format", "json"])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(
        !stdout.contains("TS2307"),
        "the baseUrl bare specifier must resolve:\n{stdout}\n{stderr}"
    );
    assert!(
        !stdout.contains("TS5102"),
        "TypeScript 5.x accepts baseUrl; the removal diagnostic is a false positive:\n{stdout}\n{stderr}"
    );
    assert!(
        stdout.contains("TS2345"),
        "the real diagnostic behind the import must surface:\n{stdout}\n{stderr}"
    );

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["errorCount"], 1, "{stdout}\n{stderr}");

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn the_base_url_removal_diagnostic_survives_on_typescript_6() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };

    let project_root = workspace_root()
        .join("target/vize-tests/tests")
        .join(format!("base-url-ts6-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&project_root);
    write_base_url_project(&project_root, "6.0.3");

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", "src", "--no-config", "--format", "json"])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    // `tsc` 6.0.3 errors on the same config (TS5101, measured), so forwarding
    // the removal verdict matches the user's own toolchain — while the import
    // keeps resolving either way.
    assert!(
        stdout.contains("TS5102"),
        "a TypeScript 6 project must keep the removal diagnostic:\n{stdout}\n{stderr}"
    );
    assert!(
        !stdout.contains("TS2307") && stdout.contains("TS2345"),
        "resolution is independent of the config verdict:\n{stdout}\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
