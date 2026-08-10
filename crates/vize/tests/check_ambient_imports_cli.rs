#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use std::{path::Path, process::Command};

#[path = "support/vue_stub.rs"]
mod vue_stub;

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn unique_case_dir() -> std::path::PathBuf {
    workspace_root()
        .join("target/vize-tests/tests")
        .join(format!("ambient-imports-{}", std::process::id()))
}

fn write_file(root: &Path, path: &str, content: &str) {
    let file_path = root.join(path);
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(file_path, content).unwrap();
}

fn resolve_test_corsa_path() -> Option<std::path::PathBuf> {
    let sibling_cache = workspace_root().parent()?.join("corsa-bind/.cache/tsgo");
    if sibling_cache.exists() {
        return Some(sibling_cache);
    }
    let workspace_bin = workspace_root().join("node_modules/.bin/tsgo");
    workspace_bin.exists().then_some(workspace_bin)
}

#[test]
fn explicit_check_registers_types_imported_by_ambient_declarations() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = unique_case_dir();
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(&project_root).unwrap();
    vue_stub::install_vue_jsx_type_stub(&project_root);

    write_file(
        &project_root,
        "package.json",
        r#"{ "name": "ambient-imports", "private": true, "type": "module" }"#,
    );
    write_file(
        &project_root,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "skipLibCheck": true,
    "noEmit": true
  },
  "include": ["src/**/*.ts", "src/**/*.d.ts", "src/**/*.vue"]
}"#,
    );
    write_file(
        &project_root,
        "src/type/globals.d.ts",
        r#"import type { WelcomeRuntime, welcomeRuntimeKey } from "../welcome/preloadType";

declare global {
  interface Window {
    readonly [welcomeRuntimeKey]: WelcomeRuntime;
  }
}
"#,
    );
    write_file(
        &project_root,
        "src/welcome/preloadType.ts",
        r#"import type { RuntimeContract } from "./runtimeContract";

export const welcomeRuntimeKey = "welcomeRuntime";
export type WelcomeRuntime = RuntimeContract;
"#,
    );
    write_file(
        &project_root,
        "src/welcome/runtimeContract.ts",
        "export type RuntimeContract = { ping(): string };\n",
    );
    write_file(
        &project_root,
        "src/App.vue",
        r#"<script setup lang="ts">
const result: string = window.welcomeRuntime.ping();
</script>

<template>
  <div>{{ result }}</div>
</template>
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "src/App.vue",
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
        "ambient import check failed:\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["errorCount"], 0, "{stdout}\n{stderr}");
    assert_eq!(json["warningCount"], 0, "{stdout}\n{stderr}");
    assert_eq!(
        json["fileCount"], 1,
        "supporting types must not be reported"
    );

    let generated: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            vize_canon::project_virtual_root(&project_root).join("tsconfig.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let includes = generated["include"].as_array().unwrap();
    for expected in [
        "src/type/globals.d.cts",
        "src/welcome/preloadType.ts",
        "src/welcome/runtimeContract.ts",
    ] {
        assert!(
            includes.iter().any(|value| value == expected),
            "missing {expected} from generated project: {includes:?}"
        );
    }

    let _ = std::fs::remove_dir_all(&project_root);
}
