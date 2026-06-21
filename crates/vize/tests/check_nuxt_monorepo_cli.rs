use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[test]
fn check_explicit_nuxt_app_tsconfig_in_monorepo_keeps_app_root_detection() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_project("monorepo-nuxt-app-tsconfig");

    write_file(
        &project_root,
        "tsconfig.base.json",
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  }
}"#,
    );
    write_file(
        &project_root,
        "apps/volt/nuxt.config.ts",
        "export default defineNuxtConfig({})\n",
    );
    write_file(
        &project_root,
        "apps/volt/tsconfig.json",
        r#"{
  "extends": "../../tsconfig.base.json",
  "include": [
    "app.vue",
    "../../types/**/*.d.ts"
  ]
}"#,
    );
    write_file(
        &project_root,
        "apps/volt/app.vue",
        r#"<script setup lang="ts">
const message: GlobalMessage = { text: 'hello' }
</script>

<template>
  <NuxtLink to="/">{{ message.text }}</NuxtLink>
</template>
"#,
    );
    write_file(
        &project_root,
        "types/global.d.ts",
        r#"export {};

declare global {
  interface GlobalMessage {
    text: string;
  }
}
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "--tsconfig",
            "apps/volt/tsconfig.json",
            "apps/volt",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("failed to parse stdout as JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });
    let diagnostics = json["files"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|file| file["diagnostics"].as_array().unwrap().iter())
        .filter_map(|diagnostic| diagnostic.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(
        json["errorCount"], 0,
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.contains("NuxtLink")),
        "Nuxt app root should be detected from explicit app tsconfig: {diagnostics:#?}"
    );
    assert!(
        !stderr.contains("Failed to strip prefix from path"),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

fn create_project(name: &str) -> PathBuf {
    let project_root = workspace_root()
        .join("target")
        .join("vize-tests")
        .join(format!("{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(&project_root).unwrap();
    link_workspace_node_modules(&project_root);
    project_root
}

fn write_file(root: &Path, path: &str, content: &str) {
    let file_path = root.join(path);
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(file_path, content).unwrap();
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
        .to_path_buf()
}

fn link_workspace_node_modules(project_root: &Path) {
    let source = workspace_root().join("node_modules");
    if source.exists() {
        symlink_path(&source, &project_root.join("node_modules")).unwrap();
    }
}

fn resolve_test_corsa_path() -> Option<String> {
    if let Some(path) = std::env::var_os("CORSA_PATH") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Some(path.display().to_string());
        }
    }
    let workspace_root = workspace_root();
    [workspace_root.join("node_modules/.bin/tsgo")]
        .into_iter()
        .find(|candidate| candidate.exists())
        .map(|candidate| candidate.display().to_string())
}

fn symlink_path(source: &Path, target: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(source, target)
    }
}
