#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;
#[path = "check_allowjs_imports_cli/imported_diagnostics.rs"]
mod imported_diagnostics;
#[path = "check_allowjs_imports_cli/referenced_compiler_options.rs"]
mod referenced_compiler_options;
#[path = "check_allowjs_imports_cli/referenced_projects.rs"]
mod referenced_projects;

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use vize_carton::cstr;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
        .to_path_buf()
}

fn unique_case_dir(name: &str) -> PathBuf {
    workspace_root()
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(cstr!("check-allowjs-imports-{name}-{}", std::process::id()).as_str())
}

fn resolve_test_corsa_path() -> Option<PathBuf> {
    let root = workspace_root();
    [
        root.parent()?.join("corsa-bind/.cache/tsgo"),
        root.join("node_modules/.bin/tsgo"),
        root.join("examples/vite-musea/node_modules/.bin/tsgo"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}

fn required_corsa_path() -> Option<PathBuf> {
    corsa_requirement::required_or_skip(resolve_test_corsa_path())
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

#[test]
fn check_allowjs_resolves_project_local_js_imports() {
    let Some(corsa_path) = required_corsa_path() else {
        return;
    };
    let project_root = unique_case_dir("local-js");
    let _ = std::fs::remove_dir_all(&project_root);

    write(
        &project_root,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "allowJs": true,
    "checkJs": false,
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["lint/**/*.ts", "lint/**/*.js", ".eslintrc.js"]
}"#,
    );
    write(
        &project_root,
        ".eslintrc.js",
        r#"export const parserOptions = { parser: "vue-eslint-parser" };
"#,
    );
    write(
        &project_root,
        "lint/rules/no-access-process.js",
        r#"export default {
  meta: { messages: { unexpected: "Do not access process directly." } },
};
"#,
    );
    write(
        &project_root,
        "lint/__tests__/no-access-process.spec.ts",
        r#"import { parserOptions } from "../../.eslintrc.js";
import rule from "../rules/no-access-process";

const parser: string = parserOptions.parser;
const message: string = rule.meta.messages.unexpected;
void parser;
void message;
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", &corsa_path)
        .args([
            "check",
            "--no-config",
            "--tsconfig",
            "tsconfig.json",
            "lint/__tests__/no-access-process.spec.ts",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "check failed:\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("invalid JSON ({error}):\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });
    assert_eq!(json["errorCount"], serde_json::json!(0), "{stdout}");
    assert!(
        !stdout.contains("TS2307"),
        "project-local JS imports should resolve under allowJs:\n{stdout}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn default_check_reports_errors_from_allowjs_project_roots() {
    let Some(corsa_path) = required_corsa_path() else {
        return;
    };
    let project_root = unique_case_dir("default-root-diagnostic");
    let _ = std::fs::remove_dir_all(&project_root);

    write(
        &project_root,
        "tsconfig.base.json",
        r#"{
  "compilerOptions": {
    "allowJs": true,
    "checkJs": true,
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  }
}"#,
    );
    write(
        &project_root,
        "tsconfig.json",
        r#"{ "extends": "./tsconfig.base.json", "include": ["src/**/*"] }"#,
    );
    write(
        &project_root,
        "src/invalid.js",
        r#"/** @type {string} */
export const message = 42;
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", &corsa_path)
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

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("invalid JSON ({error}):\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });
    assert_eq!(json["errorCount"], serde_json::json!(1), "{stdout}");
    assert_eq!(
        json["files"][0]["file"],
        serde_json::json!("src/invalid.js"),
        "{stdout}"
    );
    assert!(
        json["files"][0]["diagnostics"][0]
            .as_str()
            .is_some_and(|diagnostic| diagnostic.contains("TS2322")),
        "{stdout}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn explicit_allowjs_file_reports_authored_diagnostics() {
    let Some(corsa_path) = required_corsa_path() else {
        return;
    };
    let project_root = unique_case_dir("explicit-root-diagnostic");
    let _ = std::fs::remove_dir_all(&project_root);

    write(
        &project_root,
        "tsconfig.base.json",
        r#"{
  "compilerOptions": {
    "allowJs": true,
    "checkJs": true,
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  }
}"#,
    );
    write(
        &project_root,
        "tsconfig.json",
        r#"{ "extends": "./tsconfig.base.json", "include": ["src/**/*"] }"#,
    );
    write(
        &project_root,
        "src/invalid.js",
        r#"/** @type {string} */
export const message = 42;
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", &corsa_path)
        .args([
            "check",
            "--no-config",
            "--tsconfig",
            "tsconfig.json",
            "src/invalid.js",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("invalid JSON ({error}):\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });
    assert_eq!(json["errorCount"], serde_json::json!(1), "{stdout}");
    assert_eq!(
        json["files"][0]["file"],
        serde_json::json!("src/invalid.js"),
        "{stdout}"
    );
    assert!(
        json["files"][0]["diagnostics"][0]
            .as_str()
            .is_some_and(|diagnostic| diagnostic.contains("TS2322")),
        "{stdout}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
