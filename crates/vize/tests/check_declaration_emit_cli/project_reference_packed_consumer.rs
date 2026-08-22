use super::*;

#[test]
fn project_reference_declarations_typecheck_from_packed_consumer() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let case_root = unique_case_dir("packed-project-reference-declarations");
    let _ = std::fs::remove_dir_all(&case_root);
    let ui_root = case_root.join("packages/ui");
    let app_root = case_root.join("packages/app");
    std::fs::create_dir_all(ui_root.join("src")).unwrap();
    std::fs::create_dir_all(app_root.join("src")).unwrap();
    write_project_reference_library(&ui_root, None);
    write_project_reference_library(&app_root, Some("../ui"));
    vue_stub::install_vue_jsx_type_stub(&ui_root);
    vue_stub::install_vue_jsx_type_stub(&app_root);
    std::fs::write(
        ui_root.join("package.json"),
        r#"{
  "name": "@scope/ref-ui",
  "types": "./src/index.ts",
  "exports": { ".": { "types": "./src/index.ts", "default": "./src/index.ts" } }
}"#,
    )
    .unwrap();
    std::fs::write(
        ui_root.join("src/Button.vue"),
        r#"<script setup lang="ts">
export interface ButtonProps {
  count: number
}
defineProps<ButtonProps>()
</script>
<template>{{ count }}</template>
"#,
    )
    .unwrap();
    std::fs::write(
        ui_root.join("src/index.ts"),
        r#"export { default as Button } from "./Button.vue";
export type { ButtonProps } from "./Button.vue";
"#,
    )
    .unwrap();
    std::fs::write(
        app_root.join("package.json"),
        r#"{
  "name": "@scope/ref-app",
  "types": "./src/index.ts",
  "exports": { ".": { "types": "./src/index.ts", "default": "./src/index.ts" } }
}"#,
    )
    .unwrap();
    std::fs::write(
        app_root.join("src/index.ts"),
        r#"export { Button } from "@scope/ref-ui";
export type { ButtonProps } from "@scope/ref-ui";
"#,
    )
    .unwrap();
    link_project_package(&ui_root, &app_root.join("node_modules/@scope/ref-ui"));

    emit_declarations(&ui_root, corsa_path.as_str());
    emit_declarations(&app_root, corsa_path.as_str());

    let consumer_root = create_cli_project(
        "packed-project-reference-consumer",
        &[(
            "src/index.ts",
            r#"import { Button } from "@scope/ref-app";
import type { ButtonProps } from "@scope/ref-app";

type Props = InstanceType<typeof Button>["$props"];
const componentProps: Props = { count: 1 };
const exportedProps: ButtonProps = componentProps;
void exportedProps;
"#,
        )],
    );
    vue_stub::install_vue_jsx_type_stub(&consumer_root);
    install_packed_package(
        &ui_root.join("types"),
        &consumer_root.join("node_modules/@scope/ref-ui"),
        "@scope/ref-ui",
    );
    install_packed_package(
        &app_root.join("types"),
        &consumer_root.join("node_modules/@scope/ref-app"),
        "@scope/ref-app",
    );

    let clean_output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&consumer_root)
        .env("CORSA_PATH", corsa_path.as_str())
        .args(["check", ".", "--format", "json"])
        .output()
        .unwrap();
    let clean_stdout = std::str::from_utf8(&clean_output.stdout).unwrap();
    let clean_stderr = std::str::from_utf8(&clean_output.stderr).unwrap();
    assert_eq!(
        clean_output.status.code(),
        Some(0),
        "project-reference packed consumer should typecheck from declarations only:\n{clean_stdout}\n{clean_stderr}"
    );

    std::fs::write(
        consumer_root.join("src/index.ts"),
        r#"import { Button } from "@scope/ref-app";

type Props = InstanceType<typeof Button>["$props"];
const componentProps: Props = { count: "bad" };
void componentProps;
"#,
    )
    .unwrap();
    let broken_output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&consumer_root)
        .env("CORSA_PATH", corsa_path.as_str())
        .args(["check", ".", "--format", "json"])
        .output()
        .unwrap();
    let broken_stdout = std::str::from_utf8(&broken_output.stdout).unwrap();
    let broken_stderr = std::str::from_utf8(&broken_output.stderr).unwrap();
    assert_eq!(
        broken_output.status.code(),
        Some(1),
        "project-reference packed consumer should report authored errors:\n{broken_stdout}\n{broken_stderr}"
    );
    let broken_json: serde_json::Value = serde_json::from_str(broken_stdout).unwrap();
    assert_eq!(
        broken_json["errorCount"], 1,
        "{broken_stdout}\n{broken_stderr}"
    );
    let consumer_file = broken_json["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|file| file["file"] == "src/index.ts")
        .unwrap_or_else(|| {
            panic!("missing authored consumer source:\n{broken_stdout}\n{broken_stderr}")
        });
    assert_eq!(
        consumer_file["diagnostics"],
        serde_json::json!([
            "error:4:33 [TS2322] Type 'string' is not assignable to type 'number'."
        ]),
        "project-reference packed consumer diagnostic should stay authored:\n{broken_stdout}\n{broken_stderr}"
    );

    let _ = std::fs::remove_dir_all(case_root);
    let _ = std::fs::remove_dir_all(consumer_root);
}

fn write_project_reference_library(root: &Path, reference: Option<&str>) {
    let references = reference
        .map(|path| format!(r#","references":[{{"path":"{path}"}}]"#))
        .unwrap_or_default();
    std::fs::write(
        root.join("tsconfig.json"),
        format!(
            r#"{{
  "compilerOptions": {{
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "composite": true,
    "declaration": true,
    "declarationMap": true,
    "noEmit": true
  }},
  "include": ["src/**/*"]{references}
}}"#
        ),
    )
    .unwrap();
}

fn emit_declarations(project_root: &Path, corsa_path: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            ".",
            "--format",
            "json",
            "--declaration",
            "--declaration-dir",
            "types",
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
}

fn install_packed_package(declarations: &Path, target: &Path, name: &str) {
    let types = if declarations.join("index.d.ts").is_file() {
        "./index.d.ts"
    } else {
        "./src/index.d.ts"
    };
    std::fs::create_dir_all(target).unwrap();
    std::fs::write(
        target.join("package.json"),
        format!(
            r#"{{
  "name": "{name}",
  "types": "{types}",
  "exports": {{ ".": {{ "types": "{types}", "default": "./index.js" }} }}
}}"#
        ),
    )
    .unwrap();
    copy_tree(declarations, target);
}

fn copy_tree(source: &Path, target: &Path) {
    std::fs::create_dir_all(target).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if source_path.is_dir() {
            copy_tree(&source_path, &target_path);
        } else {
            std::fs::copy(&source_path, &target_path).unwrap();
        }
    }
}

fn link_project_package(source: &Path, target: &Path) {
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(source, target).unwrap();
}
