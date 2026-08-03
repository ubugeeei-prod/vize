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

fn unique_case_dir(name: &str) -> std::path::PathBuf {
    static NEXT_CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    workspace_root()
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(format!("{name}-{}-{case_id}", std::process::id()))
}

fn link_workspace_node_modules(project_root: &Path) {
    let source = workspace_root().join("node_modules");
    let target = project_root.join("node_modules");
    if target.exists() {
        return;
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(source, target).unwrap();
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

#[test]
fn check_from_package_cwd_uses_package_local_tsconfig_inputs() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };

    let workspace = unique_case_dir("package-cwd-check");
    let _ = std::fs::remove_dir_all(&workspace);
    std::fs::create_dir_all(&workspace).unwrap();
    link_workspace_node_modules(&workspace);

    write_file(
        &workspace,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "paths": {
      "~/*": ["*"]
    },
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    );
    write_file(
        &workspace,
        "src/generated/tecack/custom.ts",
        "export const rootOnly: string = 'root';\n",
    );

    let package_root = workspace.join("devtools");
    write_file(
        &package_root,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "extends": "../tsconfig.json",
  "include": ["src/**/*.ts", "src/**/*.vue"]
}"#,
    );
    write_file(
        &package_root,
        "src/App.vue",
        r#"<script setup lang="ts">
import { rootOnly } from "~/src/generated/tecack/custom";

const msg: string = "ok";
void rootOnly;
</script>

<template>{{ msg }}</template>
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&package_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "src",
            "--no-config",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "package-local check failed:\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(
        json["errorCount"], 0,
        "unexpected diagnostics:\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(json["warningCount"], 0, "{stdout}\n{stderr}");
    assert_eq!(json["fileCount"], 1, "{stdout}\n{stderr}");
    let files = json["files"].as_array().unwrap();
    assert_eq!(files.len(), 1, "{stdout}\n{stderr}");
    assert_eq!(files[0]["file"], "src/App.vue", "{stdout}\n{stderr}");
    assert_eq!(
        files[0]["diagnostics"],
        serde_json::json!([]),
        "{stdout}\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&workspace);
}

#[test]
fn check_from_workspace_root_accepts_subproject_directory_input() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };

    let workspace = unique_case_dir("workspace-root-subproject-dir");
    let _ = std::fs::remove_dir_all(&workspace);
    std::fs::create_dir_all(&workspace).unwrap();
    link_workspace_node_modules(&workspace);

    write_file(
        &workspace,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["types/**/*.d.ts"]
}"#,
    );
    write_file(
        &workspace,
        "types/root.d.ts",
        r#"export {};
declare global {
  const rootValue: string;
}
"#,
    );
    write_file(
        &workspace,
        "devtools/tsconfig.json",
        r#"{
  "extends": "../tsconfig.json",
  "include": ["src/**/*.vue", "../types/**/*.d.ts"]
}"#,
    );
    write_file(
        &workspace,
        "devtools/src/App.vue",
        r#"<script setup lang="ts">
const msg: string = rootValue;
</script>

<template>{{ msg }}</template>
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&workspace)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "--no-config",
            "--tsconfig",
            "devtools/tsconfig.json",
            "devtools/src",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "workspace-root subproject check failed:\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        !stderr.contains("Failed to strip prefix from path"),
        "subproject directory inputs must not crash during virtual path mirroring:\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["errorCount"], 0, "{stdout}\n{stderr}");
    assert_eq!(json["warningCount"], 0, "{stdout}\n{stderr}");
    assert_eq!(json["fileCount"], 1, "{stdout}\n{stderr}");
    assert_eq!(json["files"][0]["file"], "devtools/src/App.vue");

    let _ = std::fs::remove_dir_all(&workspace);
}

#[test]
fn check_from_package_cwd_resolves_package_local_dependencies() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };

    let workspace = unique_case_dir("package-cwd-local-deps");
    let _ = std::fs::remove_dir_all(&workspace);
    std::fs::create_dir_all(&workspace).unwrap();
    write_file(
        &workspace,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": []
}"#,
    );

    let package_root = workspace.join("test/e2e/onepass/sign-in");
    write_file(
        &package_root,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    );
    write_file(
        &package_root,
        "node_modules/child-only/package.json",
        r#"{ "name": "child-only", "version": "1.0.0", "types": "index.d.ts" }"#,
    );
    write_file(
        &package_root,
        "node_modules/child-only/index.d.ts",
        "export declare const childOnly: string;\n",
    );
    write_file(
        &package_root,
        "src/main.ts",
        r#"import { childOnly } from "child-only";

const value: string = childOnly;
void value;
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&package_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "src",
            "--no-config",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "package-local dependency check failed:\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["errorCount"], 0, "{stdout}\n{stderr}");
    assert!(
        !stdout.contains("Cannot find module 'child-only'"),
        "child package dependency should resolve from the package cwd:\n{stdout}\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&workspace);
}

#[test]
fn check_from_package_cwd_keeps_local_tsconfig_for_external_relative_imports() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };

    let workspace = unique_case_dir("package-cwd-external-relative-import");
    let _ = std::fs::remove_dir_all(&workspace);
    std::fs::create_dir_all(&workspace).unwrap();
    write_file(
        &workspace,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "target": "ES2018",
    "module": "ESNext",
    "moduleResolution": "Node",
    "baseUrl": ".",
    "strict": true,
    "noEmit": true
  },
  "exclude": ["node_modules", "pkg"]
}"#,
    );
    write_file(
        &workspace,
        "shared/types.ts",
        "export type Shared = { id: number };\n",
    );

    let package_root = workspace.join("pkg");
    write_file(&package_root, "package.json", r#"{ "name": "pkg" }"#);
    write_file(
        &package_root,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "strict": true,
    "noEmit": true,
    "skipLibCheck": true
  },
  "include": ["src/**/*.ts"]
}"#,
    );
    write_file(
        &package_root,
        "node_modules/exports-only/package.json",
        r#"{
  "name": "exports-only",
  "version": "1.0.0",
  "type": "module",
  "exports": {
    ".": {
      "types": "./dist/index.d.ts",
      "import": "./dist/index.js"
    }
  }
}"#,
    );
    write_file(
        &package_root,
        "node_modules/exports-only/dist/index.d.ts",
        "export declare function hello(): string;\n",
    );
    write_file(
        &package_root,
        "src/main.ts",
        r#"import { hello } from "exports-only";
import type { Shared } from "../../shared/types";

export const greeting: string = hello();
export const shared: Shared = { id: 1 };
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&package_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", "--no-config", "--format", "json"])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "package-local tsconfig was not authoritative:\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stderr.contains(&format!(
            "Building Corsa virtual project for 2 files under {}...",
            package_root.display()
        )),
        "the invocation project root should remain package-local:\n{stderr}"
    );
    assert!(
        !stdout.contains("TS2307")
            && !stdout.contains("TS5102")
            && !stdout.contains("TS5108")
            && !stdout.contains(&workspace.join("tsconfig.json").display().to_string()),
        "the outer tsconfig must not affect package diagnostics:\n{stdout}\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["errorCount"], 0, "{stdout}\n{stderr}");
    assert_eq!(json["warningCount"], 0, "{stdout}\n{stderr}");
    assert_eq!(json["fileCount"], 1, "{stdout}\n{stderr}");
    assert_eq!(
        json["files"][0]["file"], "src/main.ts",
        "{stdout}\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&workspace);
}
