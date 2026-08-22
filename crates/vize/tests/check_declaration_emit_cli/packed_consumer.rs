use super::*;

#[test]
fn emitted_vue_declarations_typecheck_from_packed_consumer() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let library_root = create_cli_project(
        "packed-declaration-library",
        &[
            (
                "src/App.vue",
                r#"<script setup lang="ts">
export interface PublicProps {
  label: string
}

defineProps<PublicProps>()
</script>

<template>
  <p>{{ label }}</p>
</template>
"#,
            ),
            (
                "src/index.ts",
                r#"export { default as App } from "./App.vue";
export type { PublicProps } from "./App.vue";
"#,
            ),
        ],
    );
    vue_stub::install_vue_jsx_type_stub(&library_root);

    let emit_output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&library_root)
        .env("CORSA_PATH", corsa_path.as_str())
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
    let emit_stdout = std::str::from_utf8(&emit_output.stdout).unwrap();
    let emit_stderr = std::str::from_utf8(&emit_output.stderr).unwrap();
    assert_eq!(
        emit_output.status.code(),
        Some(0),
        "stdout:\n{emit_stdout}\nstderr:\n{emit_stderr}"
    );

    let consumer_root = create_cli_project(
        "packed-declaration-consumer",
        &[(
            "src/index.ts",
            r#"import { App } from "@scope/emitted-vue";
import type { PublicProps } from "@scope/emitted-vue";

type Props = InstanceType<typeof App>["$props"];
const componentProps: Props = { label: "ok" };
const exportedProps: PublicProps = componentProps;
void exportedProps;
"#,
        )],
    );
    vue_stub::install_vue_jsx_type_stub(&consumer_root);

    let package_root = consumer_root.join("node_modules/@scope/emitted-vue");
    std::fs::create_dir_all(&package_root).unwrap();
    std::fs::write(
        package_root.join("package.json"),
        r#"{
  "name": "@scope/emitted-vue",
  "types": "./index.d.ts",
  "exports": {
    ".": {
      "types": "./index.d.ts",
      "default": "./index.js"
    }
  }
}"#,
    )
    .unwrap();
    copy_tree(&library_root.join("types"), &package_root);

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
        "packed consumer should typecheck from declarations only:\n{clean_stdout}\n{clean_stderr}"
    );

    std::fs::write(
        consumer_root.join("src/index.ts"),
        r#"import { App } from "@scope/emitted-vue";

type Props = InstanceType<typeof App>["$props"];
const componentProps: Props = { label: 42 };
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
        "packed consumer should report authored declaration-consumer errors:\n{broken_stdout}\n{broken_stderr}"
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
            "error:4:33 [TS2322] Type 'number' is not assignable to type 'string'."
        ]),
        "packed consumer diagnostic should point at authored consumer source:\n{broken_stdout}\n{broken_stderr}"
    );

    let _ = std::fs::remove_dir_all(&library_root);
    let _ = std::fs::remove_dir_all(&consumer_root);
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
