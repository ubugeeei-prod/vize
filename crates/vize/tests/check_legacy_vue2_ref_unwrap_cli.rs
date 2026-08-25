#![cfg(feature = "legacy")]

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use vize_s0::cstr;

#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

#[test]
fn vue27_plain_value_objects_are_not_ref_unwrapped() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_project();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
            "src/App.vue",
        ])
        .output()
        .unwrap();

    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(stdout).unwrap();
    assert_eq!(json["fileCount"], 1, "stdout:\n{stdout}\nstderr:\n{stderr}");
    assert_eq!(
        json["errorCount"], 0,
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    for unexpected in ["TS2551", "Property 'value' does not exist"] {
        assert!(
            !stdout.contains(unexpected),
            "plain value object was mistaken for a ref ({unexpected}):\n{stdout}"
        );
    }

    let _ = std::fs::remove_dir_all(&project_root);
}

fn create_project() -> PathBuf {
    let project_root = workspace_root()
        .join("target/vize-tests/tests")
        .join(cstr!("vue27-plain-value-ref-unwrap-{}", std::process::id()).as_str());
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    write_vue27_stub(&project_root.join("node_modules")).unwrap();
    write_vite_stub(&project_root.join("node_modules")).unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*.vue"]
}"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("vize.config.json"),
        r#"{ "compiler": { "compatibility": { "vueVersion": "2.7" } } }"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/App.vue"),
        r#"<script setup lang="ts">
import { ref } from 'vue'

const OPTION = { text: 'Login info', value: 'LOGIN_INFO' } as const
const count = ref(0)
const wantsString = (value: string) => value
const wantsNumber = (value: number) => value
</script>

<template>
  <div>{{ wantsString(OPTION.value) }} {{ wantsNumber(count) }}</div>
</template>
"#,
    )
    .unwrap();
    project_root
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn resolve_test_corsa_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("CORSA_PATH")
        && Path::new(&path).exists()
    {
        return Some(path.into());
    }
    [
        workspace_root().join("node_modules/.bin/tsgo"),
        workspace_root().join("examples/vite-musea/node_modules/.bin/tsgo"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}

fn write_vue27_stub(target: &Path) -> std::io::Result<()> {
    let vue_types = target.join("vue/types");
    std::fs::create_dir_all(&vue_types)?;
    std::fs::write(
        target.join("vue/package.json"),
        r#"{ "name": "vue", "types": "types/index.d.ts" }"#,
    )?;
    std::fs::write(
        vue_types.join("index.d.ts"),
        r#"export interface Ref<T> { value: T }
export declare function ref<T>(value: T): Ref<T>;
export default { version: '2.7.16' };
"#,
    )
}

fn write_vite_stub(target: &Path) -> std::io::Result<()> {
    let vite = target.join("vite");
    std::fs::create_dir_all(&vite)?;
    std::fs::write(
        vite.join("package.json"),
        r#"{ "name": "vite", "types": "client.d.ts" }"#,
    )?;
    std::fs::write(vite.join("client.d.ts"), "")
}
