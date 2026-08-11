//! Native TypeScript module-resolution authority for package Vue sources (#4002).

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

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn resolve_test_corsa_path() -> Option<String> {
    corsa_path::resolve(workspace_root())
}

fn case_dir(name: &str) -> std::path::PathBuf {
    workspace_root()
        .join("target/vize-tests/tests")
        .join(format!("package-resolution-{name}-{}", std::process::id()))
}

fn write_file(root: &Path, path: &str, content: &str) {
    let file = root.join(path);
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(file, content).unwrap();
}

fn component(prop: &str, ty: &str) -> String {
    format!("<script setup lang=\"ts\">defineProps<{{ {prop}: {ty} }}>()</script>\n")
}

fn run_check(project: &Path, corsa: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project)
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
    assert!(
        output.status.success(),
        "native package resolution failed:\n{stdout}\n{stderr}"
    );
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(report["errorCount"], 0, "{stdout}\n{stderr}");
}

#[test]
fn node10_ignores_exports_and_maps_legacy_main_to_authored_vue() {
    let Some(corsa) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project = case_dir("node10");
    let _ = std::fs::remove_dir_all(&project);
    write_file(
        &project,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "CommonJS",
    "moduleResolution": "Node10",
    "allowArbitraryExtensions": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "noEmit": true
  },
  "include": ["src/**/*.ts"]
}"#,
    );
    write_file(
        &project,
        "node_modules/@scope/mode/package.json",
        r#"{
  "name": "@scope/mode",
  "main": "./dist/index.js",
  "exports": { ".": "./dist/export.js" }
}"#,
    );
    write_file(
        &project,
        "node_modules/@scope/mode/dist/index.vue",
        &component("legacyOnly", "string"),
    );
    write_file(
        &project,
        "node_modules/@scope/mode/dist/export.vue",
        &component("exportsOnly", "number"),
    );
    write_file(
        &project,
        "src/entry.ts",
        r#"import Widget from "@scope/mode"
type Props = InstanceType<typeof Widget>["$props"]
type HasLegacy = "legacyOnly" extends keyof Props ? true : false
type HasExports = "exportsOnly" extends keyof Props ? true : false
export const props: Props = { legacyOnly: "native-node10" }
export const hasLegacy: HasLegacy = true
export const hasExports: HasExports = false
"#,
    );

    run_check(&project, &corsa);
    let _ = std::fs::remove_dir_all(&project);
}

#[test]
fn node10_leaves_types_versions_range_and_target_selection_to_tsgo() {
    let Some(corsa) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project = case_dir("node10-types-versions");
    let _ = std::fs::remove_dir_all(&project);
    write_file(
        &project,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "CommonJS",
    "moduleResolution": "Node10",
    "allowArbitraryExtensions": true,
    "skipLibCheck": true,
    "noEmit": true
  },
  "include": ["src/**/*.ts"]
}"#,
    );
    write_file(
        &project,
        "node_modules/@scope/oracle/package.json",
        r#"{
  "name": "@scope/oracle",
  "types": "./dist/fallback.js",
  "main": "./dist/main.js",
  "exports": { ".": "./dist/export.js" },
  "typesVersions": {
    ">=7.0": { "*": ["ts7/*"] },
    "*": { "*": ["legacy/*"] }
  }
}"#,
    );
    for (path, prop) in [
        ("ts7/dist/fallback.vue", "ts7Only"),
        ("legacy/dist/fallback.vue", "legacyOnly"),
        ("dist/fallback.vue", "fallbackOnly"),
        ("dist/main.vue", "mainOnly"),
        ("dist/export.vue", "exportsOnly"),
    ] {
        write_file(
            &project,
            &format!("node_modules/@scope/oracle/{path}"),
            &component(prop, "string"),
        );
    }
    write_file(
        &project,
        "src/types-versions.ts",
        r#"import Widget from "@scope/oracle"
type Props = InstanceType<typeof Widget>["$props"]
type Has<K extends PropertyKey> = K extends keyof Props ? true : false
export const props: Props = { ts7Only: "native-range" }
export const hasCurrent: Has<"ts7Only"> = true
export const hasLegacy: Has<"legacyOnly"> = false
export const hasFallback: Has<"fallbackOnly"> = false
export const hasMain: Has<"mainOnly"> = false
export const hasExports: Has<"exportsOnly"> = false
"#,
    );

    run_check(&project, &corsa);
    let _ = std::fs::remove_dir_all(&project);
}

fn assert_modern_mode(corsa: &str, mode: &str) {
    let project = case_dir(&mode.to_ascii_lowercase());
    let _ = std::fs::remove_dir_all(&project);
    write_file(
        &project,
        "tsconfig.json",
        &format!(
            r#"{{
  "compilerOptions": {{
    "strict": true,
    "target": "ES2022",
    "module": "{mode}",
    "moduleResolution": "{mode}",
    "allowArbitraryExtensions": true,
    "skipLibCheck": true,
    "noEmit": true
  }},
  "include": ["src/**/*"]
}}"#
        ),
    );
    write_file(
        &project,
        "node_modules/@scope/mode/package.json",
        r#"{
  "name": "@scope/mode",
  "type": "module",
  "exports": {
    ".": {
      "import": "./dist/import.js",
      "require": "./dist/require.cjs"
    }
  }
}"#,
    );
    write_file(
        &project,
        "node_modules/@scope/mode/dist/import.vue",
        &component("importOnly", "string"),
    );
    write_file(
        &project,
        "node_modules/@scope/mode/dist/require.vue",
        &component("requireOnly", "number"),
    );
    write_file(
        &project,
        "src/importer.mts",
        r#"import Widget from "@scope/mode"
type Props = InstanceType<typeof Widget>["$props"]
type HasImport = "importOnly" extends keyof Props ? true : false
type HasRequire = "requireOnly" extends keyof Props ? true : false
export const props: Props = { importOnly: "native-import" }
export const hasImport: HasImport = true
export const hasRequire: HasRequire = false
"#,
    );
    write_file(
        &project,
        "src/requirer.cts",
        r#"import Widget = require("@scope/mode")
type Component = (typeof Widget)["default"]
type Props = InstanceType<Component>["$props"]
type HasRequire = "requireOnly" extends keyof Props ? true : false
type HasImport = "importOnly" extends keyof Props ? true : false
export const props: Props = { requireOnly: 16 }
export const hasRequire: HasRequire = true
export const hasImport: HasImport = false
"#,
    );
    write_file(
        &project,
        "src/dynamic.cts",
        r#"export async function load() {
  const { default: Widget } = await import("@scope/mode")
  type Props = InstanceType<typeof Widget>["$props"]
  type HasImport = "importOnly" extends keyof Props ? true : false
  type HasRequire = "requireOnly" extends keyof Props ? true : false
  const props: Props = { importOnly: "dynamic-import" }
  const hasImport: HasImport = true
  const hasRequire: HasRequire = false
  return { props, hasImport, hasRequire }
}
"#,
    );
    write_file(
        &project,
        "src/explicit.cts",
        r#"import type Widget from "@scope/mode" with { "resolution-mode": "import" }
type Props = InstanceType<typeof Widget>["$props"]
type HasImport = "importOnly" extends keyof Props ? true : false
type HasRequire = "requireOnly" extends keyof Props ? true : false
export const props: Props = { importOnly: "explicit-import" }
export const hasImport: HasImport = true
export const hasRequire: HasRequire = false
"#,
    );

    run_check(&project, corsa);
    let _ = std::fs::remove_dir_all(&project);
}

#[test]
fn node16_selects_import_and_require_per_occurrence() {
    let Some(corsa) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    assert_modern_mode(&corsa, "Node16");
}

#[test]
fn nodenext_selects_import_and_require_per_occurrence() {
    let Some(corsa) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    assert_modern_mode(&corsa, "NodeNext");
}
