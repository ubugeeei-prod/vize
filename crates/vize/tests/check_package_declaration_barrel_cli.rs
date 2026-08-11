//! Installed declaration barrels must retain relative Vue package topology (#4002).

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

#[path = "support/corsa_path.rs"]
mod corsa_path;
#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use std::{path::Path, process::Command};

#[test]
fn declaration_barrel_reexporting_vue_keeps_the_exact_component_api() {
    let Some(corsa) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project = workspace_root().join("target/vize-tests/tests/package-declaration-barrel");
    let _ = std::fs::remove_dir_all(&project);
    write(
        &project.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "allowArbitraryExtensions": true,
    "skipLibCheck": true,
    "noEmit": true
  },
  "include": ["src/**/*.ts"]
}"#,
    );
    let package = project.join("node_modules/@scope/declaration-barrel");
    write(
        &package.join("package.json"),
        r#"{
  "name": "@scope/declaration-barrel",
  "exports": { ".": { "types": "./index.d.ts", "default": "./index.d.ts" } }
}"#,
    );
    write(
        &package.join("index.d.ts"),
        "export { default } from './Widget.vue'\n",
    );
    write(
        &package.join("Widget.vue"),
        "<script setup lang=\"ts\">defineProps<{ required: string }>()</script>\n",
    );
    write(
        &project.join("src/entry.ts"),
        r#"import Widget from "@scope/declaration-barrel"
type Props = InstanceType<typeof Widget>["$props"]
type HasRequired = "required" extends keyof Props ? true : false
type HasWrong = "wrong" extends keyof Props ? true : false
export const props: Props = { required: "exact" }
export const hasRequired: HasRequired = true
export const hasWrong: HasWrong = false
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project)
        .env("CORSA_PATH", corsa)
        .args([
            "check",
            "--no-config",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(output.status.success(), "{stdout}\n{stderr}");
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(report["errorCount"], 0, "{stdout}\n{stderr}");
    let _ = std::fs::remove_dir_all(&project);
}

#[test]
fn typescript_barrels_reaching_vue_through_private_imports_keep_exact_apis() {
    let Some(corsa) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project = workspace_root().join(format!(
        "target/vize-tests/tests/package-private-barrels-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&project);
    write(
        &project.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "allowArbitraryExtensions": true,
    "skipLibCheck": true,
    "noEmit": true
  },
  "include": ["src/**/*.ts"]
}"#,
    );

    let local = project.join("node_modules/@scope/local-private");
    write(
        &local.join("package.json"),
        r##"{
  "name": "@scope/local-private",
  "exports": "./index.ts",
  "imports": { "#component": "./Local.vue" }
}"##,
    );
    write(
        &local.join("index.ts"),
        "export { default } from '#component'\n",
    );
    write(
        &local.join("Local.vue"),
        "<script setup lang=\"ts\">defineProps<{ localOnly: string }>()</script>\n",
    );

    let external = project.join("node_modules/@scope/external-private");
    write(
        &external.join("package.json"),
        r##"{
  "name": "@scope/external-private",
  "exports": "./index.d.ts",
  "imports": { "#component": "@scope/external-component" }
}"##,
    );
    write(
        &external.join("index.d.ts"),
        "export { default } from '#component'\n",
    );
    let component = project.join("node_modules/@scope/external-component");
    write(
        &component.join("package.json"),
        r#"{ "name": "@scope/external-component", "exports": "./External.vue" }"#,
    );
    write(
        &component.join("External.vue"),
        "<script setup lang=\"ts\">defineProps<{ externalOnly: number }>()</script>\n",
    );

    let nested = project.join("node_modules/@scope/nested-declaration");
    write(
        &nested.join("package.json"),
        r#"{ "name": "@scope/nested-declaration", "exports": "./index.ts" }"#,
    );
    write(
        &nested.join("index.ts"),
        "export { default } from './types'\n",
    );
    write(
        &nested.join("types.d.ts"),
        "export { default } from './Nested.vue'\n",
    );
    write(
        &nested.join("Nested.vue"),
        "<script setup lang=\"ts\">defineProps<{ nestedOnly: boolean }>()</script>\n",
    );

    write(
        &project.join("src/entry.ts"),
        r#"import Local from "@scope/local-private"
import External from "@scope/external-private"
import Nested from "@scope/nested-declaration"
type LocalProps = InstanceType<typeof Local>["$props"]
type ExternalProps = InstanceType<typeof External>["$props"]
type NestedProps = InstanceType<typeof Nested>["$props"]
type LocalHasWrong = "wrong" extends keyof LocalProps ? true : false
type ExternalHasWrong = "wrong" extends keyof ExternalProps ? true : false
type NestedHasWrong = "wrong" extends keyof NestedProps ? true : false
export const local: LocalProps = { localOnly: "exact" }
export const external: ExternalProps = { externalOnly: 1 }
export const nested: NestedProps = { nestedOnly: true }
export const localHasWrong: LocalHasWrong = false
export const externalHasWrong: ExternalHasWrong = false
export const nestedHasWrong: NestedHasWrong = false
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project)
        .env("CORSA_PATH", corsa)
        .args([
            "check",
            "--no-config",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(output.status.success(), "{stdout}\n{stderr}");
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(report["errorCount"], 0, "{stdout}\n{stderr}");
    let _ = std::fs::remove_dir_all(&project);
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap()
}

fn resolve_test_corsa_path() -> Option<String> {
    corsa_path::resolve(workspace_root())
}

fn write(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}
