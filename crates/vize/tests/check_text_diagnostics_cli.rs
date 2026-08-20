#[path = "support/corsa_path.rs"]
mod corsa_path;
#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use vize_carton::cstr;

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn resolve_test_corsa_path() -> Option<String> {
    corsa_requirement::required_or_skip(corsa_path::resolve(workspace_root()))
}

fn unique_case_dir(name: &str) -> PathBuf {
    workspace_root()
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(cstr!("{name}-{}", std::process::id()).as_str())
}

fn create_cli_project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let project_root = unique_case_dir(name);
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    for (path, source) in files {
        let target = project_root.join(path);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(target, source).unwrap();
    }
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "preserve",
    "jsxImportSource": "vue",
    "noEmit": true
  },
  "include": ["src/**/*"]
}
"#,
    )
    .unwrap();
    project_root
}

fn run_check(project_root: &Path, corsa_path: &str, format: Option<&str>) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command
        .current_dir(project_root)
        .env("CORSA_PATH", corsa_path)
        .env("NO_COLOR", "1")
        .env_remove("CLICOLOR_FORCE")
        .env_remove("FORCE_COLOR")
        .args(["check", "."]);
    if let Some(format) = format {
        command.args(["--format", format]);
    }
    command.output().unwrap()
}

#[test]
fn check_json_stays_machine_oriented_while_text_includes_source_context() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "text-source-context",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
const count: string = 0;
</script>
"#,
        )],
    );

    let json_output = run_check(&project_root, &corsa_path, Some("json"));
    let json_stdout = std::str::from_utf8(&json_output.stdout).unwrap();
    let json_stderr = std::str::from_utf8(&json_output.stderr).unwrap();
    assert_eq!(
        json_output.status.code(),
        Some(1),
        "stdout:\n{json_stdout}\nstderr:\n{json_stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(json_stdout).unwrap();
    let json_diagnostics: Vec<_> = json["files"]
        .as_array()
        .expect("JSON report should contain files")
        .iter()
        .flat_map(|file| {
            file["diagnostics"]
                .as_array()
                .expect("JSON report files should contain diagnostics")
        })
        .filter_map(serde_json::Value::as_str)
        .collect();
    assert!(
        json_diagnostics.iter().any(|diagnostic| {
            diagnostic.contains("error:2:7 [TS2322]")
                && diagnostic.contains("Type 'number' is not assignable to type 'string'")
        }),
        "JSON diagnostics should include the known type mismatch: {json_diagnostics:#?}"
    );
    assert!(
        json_diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.contains("source:")),
        "JSON diagnostics should stay stable and machine-oriented: {json_diagnostics:#?}"
    );

    let text_output = run_check(&project_root, &corsa_path, None);
    let stdout = std::str::from_utf8(&text_output.stdout).unwrap();
    let stderr = std::str::from_utf8(&text_output.stderr).unwrap();
    assert_eq!(
        text_output.status.code(),
        Some(1),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("error:2:7 [TS2322]"),
        "text diagnostics should stay parser-friendly:\n{stdout}"
    );
    assert!(
        stdout.contains("source: const count: string = 0;"),
        "text diagnostics should include authored source context:\n{stdout}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_text_diagnostics_name_template_bindings() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "text-template-source-context",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineProps<{ modelValue: number; kind: "num"; n: number }>();
defineEmits<{ save: [id: number] }>();
defineSlots<{ default(props: { count: number }): unknown }>();
</script>
"#,
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import { ref } from "vue";
import Child from "./Child.vue";

const text = ref("bad");
</script>

<template>
  <Child v-model="text" />
  <Child kind="num" :s="'bad'" />
  <Child :model-value="1" kind="num" :n="1" @save="(id: string) => {}" />
  <Child :model-value="1" kind="num" :n="1" v-slot="{ count }">{{ count.toUpperCase() }}</Child>
</template>
"#,
            ),
        ],
    );

    let output = run_check(&project_root, &corsa_path, None);
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("binding: modelValue"),
        "v-model text diagnostics should name the modeled prop:\n{stdout}"
    );
    assert!(
        stdout.contains("binding: 's'"),
        "shorthand prop diagnostics should name the authored prop:\n{stdout}"
    );
    assert!(
        stdout.contains("binding: @save"),
        "event diagnostics should name the authored event:\n{stdout}"
    );
    assert!(
        stdout.contains("binding: #default"),
        "slot diagnostics should name the authored slot:\n{stdout}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
