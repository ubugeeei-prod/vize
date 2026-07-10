use std::{
    path::{Path, PathBuf},
    process::Command,
};

use vize_carton::cstr;

#[test]
fn check_vue_function_prop_callbacks_and_create_app_chaining() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_project();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", ".", "--format", "json", "--show-virtual-ts"])
        .output()
        .unwrap();

    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(stdout).unwrap_or_else(|error| {
        panic!("failed to parse stdout as JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });

    assert!(
        output.status.success(),
        "function prop callbacks and createApp chaining should type-check:\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(json["errorCount"], 0, "{stdout}");
    assert!(!stdout.contains("[TS7006]"), "{stdout}");
    assert!(!stdout.contains("[TS2339]"), "{stdout}");
    assert!(
        !stdout.contains("void ((value) => `${value * 100}%`); // VBind"),
        "component prop callback should only be checked through typed prop assignment:\n{stdout}"
    );

    let _ = std::fs::remove_dir_all(project_root);
}

fn create_project() -> std::path::PathBuf {
    let project_root = unique_case_dir("function-prop-create-app-chain");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src/components")).unwrap();
    link_workspace_node_modules(&project_root).unwrap();
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
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/components/Comp1.vue"),
        r#"<template>
  <p>Value: {{ textConverter(value) }}</p>
</template>

<script setup lang="ts">
import { ref } from 'vue';

defineProps<{
  textConverter: (value: number) => string;
}>();

const value = ref(0.5);
</script>
"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/App.vue"),
        r#"<script setup lang="ts">
import Comp1 from './components/Comp1.vue';
</script>

<template>
  <Comp1 :textConverter="(value) => `${value * 100}%`" />
</template>
"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/main.ts"),
        r#"import { createApp } from 'vue';
import App from './App.vue';

createApp(App).mount('#app');
"#,
    )
    .unwrap();
    project_root
}

fn unique_case_dir(name: &str) -> std::path::PathBuf {
    static NEXT_CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    workspace_root()
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(cstr!("{name}-{}-{case_id}", std::process::id()).as_str())
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn link_workspace_node_modules(project_root: &Path) -> std::io::Result<()> {
    let source = workspace_root().join("node_modules");
    let target = project_root.join("node_modules");
    if target.exists() {
        std::fs::remove_dir_all(&target)?;
    }
    symlink_dir(&source, &target)
}

#[cfg(unix)]
fn symlink_dir(source: &Path, target: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(source, target)
}

#[cfg(windows)]
fn symlink_dir(source: &Path, target: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(source, target)
}

fn resolve_test_corsa_path() -> Option<PathBuf> {
    let workspace_root = workspace_root();
    let sibling_cache = workspace_root.parent()?.join("corsa-bind/.cache/tsgo");
    if sibling_cache.exists() {
        return Some(sibling_cache);
    }

    [
        workspace_root.join("node_modules/.bin/tsgo"),
        workspace_root.join("examples/vite-musea/node_modules/.bin/tsgo"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}
