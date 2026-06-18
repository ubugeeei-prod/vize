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
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    );
    write_file(
        &workspace,
        "src/generated/tecack/custom.ts",
        "export const rootOnly: string = 1;\n",
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
  "include": ["src/**/*.ts", "src/**/*.vue"]
}"#,
    );
    write_file(
        &package_root,
        "src/App.vue",
        r#"<script setup lang="ts">
const msg: string = "ok";
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
    assert!(
        !stderr.contains("outside project root"),
        "{stdout}\n{stderr}"
    );
    assert!(!stdout.contains("rootOnly"), "{stdout}\n{stderr}");

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["errorCount"], 0, "{stdout}\n{stderr}");

    let _ = std::fs::remove_dir_all(&workspace);
}
