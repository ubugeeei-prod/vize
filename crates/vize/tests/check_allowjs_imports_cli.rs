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
    process::{Command, Output},
};

use vize_s0::cstr;

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

fn run_check(project_root: &Path, corsa_path: &Path, inputs: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", "--no-config", "--tsconfig", "tsconfig.json"])
        .args(inputs)
        .args(["--format", "json"])
        .output()
        .unwrap()
}

fn output_text(output: &Output) -> (std::string::String, std::string::String) {
    (
        std::string::String::from_utf8(output.stdout.clone()).unwrap(),
        std::string::String::from_utf8(output.stderr.clone()).unwrap(),
    )
}

fn output_json(output: &Output) -> serde_json::Value {
    let (stdout, stderr) = output_text(output);
    serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("invalid JSON ({error}):\nstdout:\n{stdout}\nstderr:\n{stderr}")
    })
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

    let output = run_check(
        &project_root,
        &corsa_path,
        &["lint/__tests__/no-access-process.spec.ts"],
    );
    let (stdout, stderr) = output_text(&output);
    assert!(
        output.status.success(),
        "check failed:\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(output_json(&output)["errorCount"], 0, "{stdout}");
    assert!(
        !stdout.contains("TS2307"),
        "project-local JS imports should resolve under allowJs:\n{stdout}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn default_check_follows_checkjs_across_broken_repaired_and_disabled_runs() {
    let Some(corsa_path) = required_corsa_path() else {
        return;
    };
    let project_root = unique_case_dir("default-root-diagnostic");
    let _ = std::fs::remove_dir_all(&project_root);
    const BASE_PREFIX: &str = r#"{
  "compilerOptions": {
    "allowJs": true,
    "checkJs": "#;
    const BASE_SUFFIX: &str = r#",
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  }
}"#;
    let config = |check_js| format!("{BASE_PREFIX}{check_js}{BASE_SUFFIX}");
    const BROKEN: &str = "/** @type {string} */\nexport const message = 42;\n";
    const REPAIRED: &str = "/** @type {string} */\nexport const message = 'ok';\n";

    write(&project_root, "tsconfig.base.json", &config(true));
    write(
        &project_root,
        "tsconfig.json",
        r#"{ "extends": "./tsconfig.base.json", "include": ["src/**/*"] }"#,
    );
    write(&project_root, "src/invalid.js", BROKEN);

    let broken = run_check(&project_root, &corsa_path, &[]);
    let (stdout, stderr) = output_text(&broken);
    assert_eq!(
        broken.status.code(),
        Some(1),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json = output_json(&broken);
    assert_eq!(json["errorCount"], 1, "{stdout}");
    assert_eq!(json["files"][0]["file"], "src/invalid.js", "{stdout}");
    assert!(
        json["files"][0]["diagnostics"][0]
            .as_str()
            .is_some_and(|diagnostic| diagnostic.contains("TS2322")),
        "{stdout}"
    );

    write(&project_root, "src/invalid.js", REPAIRED);
    let repaired = run_check(&project_root, &corsa_path, &[]);
    assert!(repaired.status.success(), "{:?}", output_text(&repaired));
    assert_eq!(output_json(&repaired)["errorCount"], 0);

    write(&project_root, "src/invalid.js", BROKEN);
    write(&project_root, "tsconfig.base.json", &config(false));
    let unchecked = run_check(&project_root, &corsa_path, &[]);
    assert!(unchecked.status.success(), "{:?}", output_text(&unchecked));
    assert_eq!(output_json(&unchecked)["errorCount"], 0);

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
        "/** @type {string} */\nexport const message = 42;\n",
    );

    let output = run_check(&project_root, &corsa_path, &["src/invalid.js"]);
    let (stdout, stderr) = output_text(&output);
    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json = output_json(&output);
    assert_eq!(json["errorCount"], 1, "{stdout}");
    assert_eq!(json["files"][0]["file"], "src/invalid.js", "{stdout}");
    assert!(
        json["files"][0]["diagnostics"][0]
            .as_str()
            .is_some_and(|diagnostic| diagnostic.contains("TS2322")),
        "{stdout}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
