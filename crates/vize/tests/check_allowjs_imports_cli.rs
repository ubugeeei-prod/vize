#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

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

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn run_project_check(project_root: &Path, corsa_path: &Path) -> std::process::Output {
    run_check(project_root, corsa_path, &[])
}

fn run_check(project_root: &Path, corsa_path: &Path, patterns: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "--no-config",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
        .args(patterns)
        .output()
        .unwrap()
}

#[test]
fn check_allowjs_resolves_project_local_js_imports() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
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

    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert!(
        output.status.success(),
        "check failed:\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(stdout).unwrap();
    assert_eq!(json["errorCount"], serde_json::json!(0), "{stdout}");
    assert!(
        !stdout.contains("TS2307"),
        "project-local JS imports should resolve under allowJs:\n{stdout}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_allowjs_checks_javascript_roots_only_when_checkjs_is_enabled() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = unique_case_dir("javascript-roots");
    let _ = std::fs::remove_dir_all(&project_root);
    const CHECK_JS_OFF: &str = r#"{
  "compilerOptions": {
    "allowJs": true,
    "checkJs": false,
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#;
    const CHECK_JS_ON: &str = r#"{
  "compilerOptions": {
    "allowJs": true,
    "checkJs": true,
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#;
    const BROKEN: &str = "/** @type {number} */\nexport const value = \"wrong\";\n";
    const REPAIRED: &str = "/** @type {number} */\nexport const value = 1;\n";
    write(&project_root, "tsconfig.json", CHECK_JS_OFF);
    write(&project_root, "src/main.js", BROKEN);

    let unchecked = run_project_check(&project_root, &corsa_path);
    let unchecked_stdout = std::str::from_utf8(&unchecked.stdout).unwrap();
    assert!(unchecked.status.success(), "{unchecked_stdout}");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(unchecked_stdout).unwrap()["errorCount"],
        0
    );

    write(&project_root, "tsconfig.json", CHECK_JS_ON);
    let broken = run_project_check(&project_root, &corsa_path);
    let broken_stdout = std::str::from_utf8(&broken.stdout).unwrap();
    assert!(!broken.status.success(), "broken JavaScript root passed");
    let broken_json: serde_json::Value = serde_json::from_str(broken_stdout).unwrap();
    assert_eq!(broken_json["errorCount"], 1, "{broken_stdout}");
    assert!(
        broken_stdout.contains("src/main.js")
            && broken_stdout.contains("TS2322")
            && broken_stdout.contains("Type 'string' is not assignable to type 'number'"),
        "{broken_stdout}"
    );

    write(&project_root, "src/main.js", REPAIRED);
    let repaired = run_project_check(&project_root, &corsa_path);
    let repaired_stdout = std::str::from_utf8(&repaired.stdout).unwrap();
    assert!(repaired.status.success(), "{repaired_stdout}");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(repaired_stdout).unwrap()["errorCount"],
        0
    );

    write(&project_root, "src/main.js", BROKEN);
    let explicit = run_check(&project_root, &corsa_path, &["src"]);
    let explicit_stdout = std::str::from_utf8(&explicit.stdout).unwrap();
    assert!(
        !explicit.status.success()
            && explicit_stdout.contains("src/main.js")
            && explicit_stdout.contains("TS2322"),
        "explicit allowJs input was not checked:\nstdout:\n{explicit_stdout}\nstderr:\n{}",
        std::str::from_utf8(&explicit.stderr).unwrap()
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
