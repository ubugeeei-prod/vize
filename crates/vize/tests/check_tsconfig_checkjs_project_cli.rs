#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
//! `vize check` on a tsconfig project that opts JavaScript into type checking
//! (`allowJs` + `checkJs`), as in a create-vue JavaScript project (#3984).
//!
//! JSDoc types in `.js` files must reach Vue scripts in the same program, and
//! `checkJs` decides whether errors inside the JavaScript files are reported,
//! exactly as it does for `tsc`.
use crate::support::corsa_path;
use crate::support::corsa_requirement;
use crate::support::vue_stub;

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use vize_l0::cstr;

const CLEAN_FORMAT: &str = r#"/**
 * @param {number} value
 * @returns {string}
 */
export function format(value) {
  return value.toFixed(1)
}

/** @type {number} */
export const precision = 1
"#;

const BROKEN_FORMAT: &str = r#"/**
 * @param {number} value
 * @returns {string}
 */
export function format(value) {
  return value.toFixed(1)
}

/** @type {number} */
export const precision = 'one'
"#;

const CLEAN_APP: &str = r#"<script setup lang="ts">
import { format } from './format.js'

const label: string = format(2)
</script>

<template>
  <p>{{ label }}</p>
</template>
"#;

const BROKEN_APP: &str = r#"<script setup lang="ts">
import { format } from './format.js'

const label: string = format('two')
</script>

<template>
  <p>{{ label }}</p>
</template>
"#;

#[test]
fn check_js_reports_javascript_errors_and_repairs() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project = ProjectCase::new("checkjs-project", true);
    project.write("src/format.js", CLEAN_FORMAT);
    project.write("src/App.vue", CLEAN_APP);

    let clean = project.check(&corsa_path);
    assert_eq!(
        clean.status,
        Some(0),
        "clean checkJs project should pass:\n{}\n{}",
        clean.stdout,
        clean.stderr
    );
    let clean_json = parse_json(&clean);
    assert_program_sources(&clean_json, &["src/App.vue", "src/format.js"]);
    assert_eq!(clean_json["errorCount"], 0, "{}", clean.stdout);

    project.write("src/format.js", BROKEN_FORMAT);
    let broken = project.check(&corsa_path);
    assert_eq!(
        broken.status,
        Some(1),
        "a JSDoc type error must fail under checkJs:\n{}\n{}",
        broken.stdout,
        broken.stderr
    );
    let broken_json = parse_json(&broken);
    assert_eq!(broken_json["errorCount"], 1, "{}", broken.stdout);
    assert_eq!(
        diagnostics_for(&broken_json, "src/format.js"),
        vec!["error:10:14 [TS2322] Type 'string' is not assignable to type 'number'."],
        "the diagnostic must stay on the authored JavaScript span:\n{}",
        broken.stdout
    );

    project.write("src/format.js", CLEAN_FORMAT);
    let repaired = project.check(&corsa_path);
    assert_eq!(
        repaired.status,
        Some(0),
        "repaired project should pass:\n{}\n{}",
        repaired.stdout,
        repaired.stderr
    );
    assert_eq!(
        parse_json(&repaired)["errorCount"],
        0,
        "{}",
        repaired.stdout
    );
}

#[test]
fn jsdoc_types_reach_vue_scripts_across_files() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project = ProjectCase::new("checkjs-vue-consumer", true);
    project.write("src/format.js", CLEAN_FORMAT);
    project.write("src/App.vue", BROKEN_APP);

    let broken = project.check(&corsa_path);
    assert_eq!(
        broken.status,
        Some(1),
        "a Vue script misusing a JSDoc-typed function must fail:\n{}\n{}",
        broken.stdout,
        broken.stderr
    );
    let broken_json = parse_json(&broken);
    assert_eq!(broken_json["errorCount"], 1, "{}", broken.stdout);
    assert_eq!(
        diagnostics_for(&broken_json, "src/App.vue"),
        vec![
            "error:4:30 [TS2345] Argument of type 'string' is not assignable to parameter of type 'number'."
        ],
        "the diagnostic must land on the authored Vue script span:\n{}",
        broken.stdout
    );

    project.write("src/App.vue", CLEAN_APP);
    let repaired = project.check(&corsa_path);
    assert_eq!(
        repaired.status,
        Some(0),
        "repaired Vue consumer should pass:\n{}\n{}",
        repaired.stdout,
        repaired.stderr
    );
}

#[test]
fn allow_js_without_check_js_keeps_javascript_errors_silent() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project = ProjectCase::new("allowjs-without-checkjs", false);
    project.write("src/format.js", BROKEN_FORMAT);
    project.write("src/App.vue", CLEAN_APP);

    let run = project.check(&corsa_path);
    assert_eq!(
        run.status,
        Some(0),
        "without checkJs, errors inside .js files are not reported:\n{}\n{}",
        run.stdout,
        run.stderr
    );
    let json = parse_json(&run);
    assert_program_sources(&json, &["src/App.vue", "src/format.js"]);
    assert_eq!(json["errorCount"], 0, "{}", run.stdout);
}

struct CheckRun {
    status: Option<i32>,
    stdout: String,
    stderr: String,
}

struct ProjectCase {
    root: PathBuf,
}

impl ProjectCase {
    fn new(name: &str, check_js: bool) -> Self {
        let case_id = cstr!("check-{name}-{}", std::process::id());
        let root = workspace_root()
            .join("target")
            .join("vize-tests")
            .join("tests")
            .join(case_id.as_str());
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src")).unwrap();
        vue_stub::install_vue_jsx_type_stub(&root);
        let case = Self { root };
        case.write(
            "tsconfig.json",
            cstr!(
                r#"{{
  "compilerOptions": {{
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowJs": true,
    "checkJs": {check_js},
    "noEmit": true
  }},
  "include": ["src/**/*"]
}}"#
            )
            .as_str(),
        );
        case
    }

    fn write(&self, path: &str, source: &str) {
        let path = self.root.join(path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, source).unwrap();
    }

    fn check(&self, corsa_path: &str) -> CheckRun {
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(&self.root)
            .env("CORSA_PATH", corsa_path)
            .args(["check", "--format", "json"])
            .output()
            .unwrap();
        CheckRun {
            status: output.status.code(),
            stdout: std::str::from_utf8(&output.stdout).unwrap().into(),
            stderr: std::str::from_utf8(&output.stderr).unwrap().into(),
        }
    }
}

impl Drop for ProjectCase {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn parse_json(run: &CheckRun) -> serde_json::Value {
    serde_json::from_str(&run.stdout).unwrap_or_else(|error| {
        panic!(
            "failed to parse check output as JSON: {error}\nstdout:\n{}\nstderr:\n{}",
            run.stdout, run.stderr
        )
    })
}

fn assert_program_sources(json: &serde_json::Value, expected: &[&str]) {
    let programs = json["programs"]
        .as_array()
        .expect("JSON output should include programs");
    assert_eq!(programs.len(), 1, "programs: {programs:#?}");
    let files = programs[0]["files"]
        .as_array()
        .expect("program should include files")
        .iter()
        .map(|file| file.as_str().unwrap())
        .filter(|file| file.starts_with("src/"))
        .collect::<Vec<_>>();
    assert_eq!(files, expected, "program source files: {files:#?}");
}

fn diagnostics_for(json: &serde_json::Value, file_name: &str) -> Vec<String> {
    json["files"]
        .as_array()
        .expect("JSON output should include files")
        .iter()
        .find(|file| file["file"] == file_name)
        .map(|file| {
            file["diagnostics"]
                .as_array()
                .expect("file should include diagnostics")
                .iter()
                .map(|diagnostic| diagnostic.as_str().unwrap().to_owned())
                .collect()
        })
        .unwrap_or_default()
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root should exist")
        .to_path_buf()
}

fn resolve_test_corsa_path() -> Option<String> {
    corsa_requirement::required_or_skip(corsa_path::resolve(&workspace_root()))
}
