#![cfg(feature = "legacy")]

use std::{path::Path, process::Command};

use vize_s0::cstr;

#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

#[test]
fn legacy_vue2_template_emit_marks_define_emits_result_as_used() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_project("legacy-vue2-template-emit-result-binding");
    std::fs::write(
        project_root.join("src/App.vue"),
        r#"<script setup lang="ts">
const emit = defineEmits<{
  (event: 'click'): void
}>()
</script>

<template>
  <button @click="$emit('click')">Click</button>
</template>
"#,
    )
    .unwrap();

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
        "template $emit should consume the defineEmits result binding\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(stdout).unwrap();
    assert_eq!(
        json["errorCount"], 0,
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn legacy_vue2_typed_emits_do_not_report_unused_loose_emit_helper() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_project("legacy-vue2-no-unused-loose-emit-helper");
    std::fs::write(
        project_root.join("src/App.vue"),
        r#"<script setup lang="ts">
interface Emits {
  (event: 'click', value: MouseEvent): void
}

const emit = defineEmits<Emits>()

function handleClick(event: MouseEvent) {
  emit('click', event)
}
</script>

<template>
  <button @click="handleClick">Click</button>
</template>
"#,
    )
    .unwrap();

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
    assert_eq!(
        json["errorCount"], 0,
        "generated loose emit helpers must not surface under noUnusedLocals:\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        !stdout.contains("__VizeVue2LooseEmitArgs"),
        "JSON diagnostics should not mention the internal helper:\n{stdout}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn legacy_vue2_no_unused_locals_still_reports_user_unused_bindings() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_project("legacy-vue2-no-unused-user-binding");
    std::fs::write(
        project_root.join("src/App.vue"),
        r#"<script setup lang="ts">
const used = 1
const unusedLocal = 2
</script>

<template>{{ used }}</template>
"#,
    )
    .unwrap();

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
        Some(1),
        "user unused locals should still fail the check\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(stdout).unwrap();
    assert_eq!(
        json["errorCount"], 1,
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("unusedLocal"),
        "expected the user unused binding diagnostic:\n{stdout}"
    );
    assert!(
        !stdout.contains("__VizeVue2LooseEmitArgs"),
        "internal helper diagnostics must stay suppressed without hiding user diagnostics:\n{stdout}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

fn create_project(name: &str) -> std::path::PathBuf {
    let project_root = unique_case_dir(name);
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    write_test_vue2_6_stub(&project_root.join("node_modules")).unwrap();
    write_test_vite_stub(&project_root.join("node_modules")).unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true,
    "noUnusedLocals": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("vize.config.json"),
        r#"{
  "vue": {
    "version": "2.7"
  },
  "typeChecker": {
    "legacyVue2": true
  }
}"#,
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

fn unique_case_dir(name: &str) -> std::path::PathBuf {
    workspace_root()
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(cstr!("{name}-{}", std::process::id()).as_str())
}

fn resolve_test_corsa_path() -> Option<std::path::PathBuf> {
    if let Ok(path) = std::env::var("CORSA_PATH")
        && Path::new(&path).exists()
    {
        return Some(path.into());
    }

    let workspace_root = workspace_root();
    [
        workspace_root.join("node_modules/.bin/tsgo"),
        workspace_root.join("examples/vite-musea/node_modules/.bin/tsgo"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}

fn write_test_vue2_6_stub(target: &Path) -> std::io::Result<()> {
    let vue_types_dir = target.join("vue").join("types");
    std::fs::create_dir_all(&vue_types_dir)?;
    std::fs::write(
        target.join("vue").join("package.json"),
        r#"{
  "name": "vue",
  "types": "types/index.d.ts"
}"#,
    )?;
    std::fs::write(
        vue_types_dir.join("index.d.ts"),
        r#"export interface Vue {
  $attrs: Record<string, unknown>;
  $refs: Record<string, any>;
  $slots: Record<string, unknown>;
  $emit: (...args: any[]) => void;
}

export type PropType<T> = { new (...args: any[]): T & {} } | { (): T } | null;

declare const VueConstructor: {
  version: string;
};

export default VueConstructor;
"#,
    )?;
    Ok(())
}

fn write_test_vite_stub(target: &Path) -> std::io::Result<()> {
    let vite_dir = target.join("vite");
    std::fs::create_dir_all(&vite_dir)?;
    std::fs::write(
        vite_dir.join("package.json"),
        r#"{
  "name": "vite",
  "types": "client.d.ts"
}"#,
    )?;
    std::fs::write(vite_dir.join("client.d.ts"), "")?;
    Ok(())
}
