//! Nearest package scope must survive into the batch mirror (#4002).

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
fn node_modes_infer_resolution_and_use_the_nearest_commonjs_package_scope() {
    let Some(corsa) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    for mode in ["Node16", "NodeNext"] {
        assert_commonjs_scope(&corsa, mode);
    }
}

fn assert_commonjs_scope(corsa: &str, mode: &str) {
    let project = workspace_root().join(format!(
        "target/vize-tests/tests/package-scope-mode-{}",
        mode.to_ascii_lowercase()
    ));
    let _ = std::fs::remove_dir_all(&project);
    write(
        &project.join("package.json"),
        "{\"name\":\"scope-mode-fixture\",\"type\":\"commonjs\"}\n",
    );
    write(
        &project.join("tsconfig.json"),
        &r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "__MODE__",
    "allowArbitraryExtensions": true,
    "skipLibCheck": true,
    "noEmit": true
  },
  "include": ["src/**/*.ts"]
}"#
        .replace("__MODE__", mode),
    );
    let package = project.join("node_modules/@scope/mode");
    write(
        &package.join("package.json"),
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
    write(
        &package.join("dist/import.vue"),
        "<script setup lang=\"ts\">defineProps<{ importOnly: string }>()</script>\n",
    );
    write(
        &package.join("dist/require.vue"),
        "<script setup lang=\"ts\">defineProps<{ requireOnly: number }>()</script>\n",
    );
    write(
        &project.join("src/entry.ts"),
        r#"import Widget from "@scope/mode"
type Props = InstanceType<typeof Widget>["$props"]
type HasRequire = "requireOnly" extends keyof Props ? true : false
type HasImport = "importOnly" extends keyof Props ? true : false
export const props: Props = { requireOnly: 16 }
export const hasRequire: HasRequire = true
export const hasImport: HasImport = false
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
