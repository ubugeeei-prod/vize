//! TSX-generating Vue package sources must not be written into `.ts` probes (#4002).

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
fn runtime_js_and_jsx_targets_forward_native_probes_to_tsx_companions() {
    let Some(corsa) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project = workspace_root().join(format!(
        "target/vize-tests/tests/package-tsx-shadow-{}",
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
    "jsx": "Preserve",
    "skipLibCheck": true,
    "noEmit": true
  },
  "include": ["src/**/*.ts"]
}"#,
    );
    let package = project.join("node_modules/@scope/tsx-shadow");
    write(
        &package.join("package.json"),
        r#"{
  "name": "@scope/tsx-shadow",
  "exports": {
    ".": "./dist/Runtime.js",
    "./jsx": "./dist/Jsx.jsx"
  }
}"#,
    );
    write(
        &package.join("dist/Runtime.vue"),
        &tsx_component("runtimeOnly"),
    );
    write(&package.join("dist/Jsx.vue"), &tsx_component("jsxOnly"));
    write(
        &project.join("src/entry.ts"),
        r#"import Runtime from "@scope/tsx-shadow"
import Jsx from "@scope/tsx-shadow/jsx"
type RuntimeProps = InstanceType<typeof Runtime>["$props"]
type JsxProps = InstanceType<typeof Jsx>["$props"]
type RuntimeHasWrong = "wrong" extends keyof RuntimeProps ? true : false
type JsxHasWrong = "wrong" extends keyof JsxProps ? true : false
export const runtime: RuntimeProps = { runtimeOnly: "tsx" }
export const jsx: JsxProps = { jsxOnly: "tsx" }
export const runtimeHasWrong: RuntimeHasWrong = false
export const jsxHasWrong: JsxHasWrong = false
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

fn tsx_component(prop: &str) -> String {
    format!(
        "<script setup lang=\"tsx\">\ndefineProps<{{ {prop}: string }}>()\nconst render = () => <div />\nvoid render\n</script>\n"
    )
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
