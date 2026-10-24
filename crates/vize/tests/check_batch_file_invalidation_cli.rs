#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
//! Repeated `vize check` runs in one project must follow file renames, deletes
//! and content changes without stale diagnostics or stale program members
//! (#3984: "file create/edit/rename/delete invalidate the correct graph").
use crate::support::corsa_path;
use crate::support::corsa_requirement;
use crate::support::vue_stub;

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use vize_l0::cstr;

const UTIL: &str = r#"export function greet(name: string): string {
  return `Hello, ${name}`
}
"#;

const APP_IMPORTING_UTIL: &str = r#"<script setup lang="ts">
import { greet } from './util'

const message: string = greet('Vize')
</script>

<template>
  <p>{{ message }}</p>
</template>
"#;

const APP_IMPORTING_HELPERS: &str = r#"<script setup lang="ts">
import { greet } from './helpers'

const message: string = greet('Vize')
</script>

<template>
  <p>{{ message }}</p>
</template>
"#;

const TYPES: &str = "export interface User {\n  name: string\n}\n";
const TYPES_CHANGED: &str = "export interface User {\n  name: number\n}\n";

const PROFILE: &str = r#"<script setup lang="ts">
import type { User } from './types'

const user: User = { name: 'Ada' }
</script>

<template>
  <p>{{ user.name }}</p>
</template>
"#;

#[test]
fn rename_without_updating_the_import_reports_the_missing_module_then_repairs() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project = ProjectCase::new("batch-rename");
    project.write("src/util.ts", UTIL);
    project.write("src/App.vue", APP_IMPORTING_UTIL);

    let clean = project.check(&corsa_path);
    assert_eq!(clean.status, Some(0), "{}\n{}", clean.stdout, clean.stderr);
    assert_program_files(&parse_json(&clean), &["src/App.vue", "src/util.ts"]);

    project.rename("src/util.ts", "src/helpers.ts");
    let stale_import = project.check(&corsa_path);
    assert_eq!(
        stale_import.status,
        Some(1),
        "{}\n{}",
        stale_import.stdout,
        stale_import.stderr
    );
    let json = parse_json(&stale_import);
    assert_program_files(&json, &["src/App.vue", "src/helpers.ts"]);
    assert_eq!(
        diagnostics_for(&json, "src/App.vue"),
        vec![
            "error:2:23 [TS2307] Cannot find module './util' or its corresponding type declarations."
        ],
        "{}",
        stale_import.stdout
    );

    project.write("src/App.vue", APP_IMPORTING_HELPERS);
    let repaired = project.check(&corsa_path);
    assert_eq!(
        repaired.status,
        Some(0),
        "{}\n{}",
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
fn delete_and_recreate_a_dependency_rechecks_its_new_content() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project = ProjectCase::new("batch-delete");
    project.write("src/types.ts", TYPES);
    project.write("src/Profile.vue", PROFILE);

    let clean = project.check(&corsa_path);
    assert_eq!(clean.status, Some(0), "{}\n{}", clean.stdout, clean.stderr);

    project.remove("src/types.ts");
    let deleted = project.check(&corsa_path);
    assert_eq!(
        deleted.status,
        Some(1),
        "{}\n{}",
        deleted.stdout,
        deleted.stderr
    );
    let json = parse_json(&deleted);
    assert_program_files(&json, &["src/Profile.vue"]);
    assert_eq!(
        diagnostics_for(&json, "src/Profile.vue"),
        vec![
            "error:2:27 [TS2307] Cannot find module './types' or its corresponding type declarations."
        ],
        "{}",
        deleted.stdout
    );

    project.write("src/types.ts", TYPES_CHANGED);
    let changed = project.check(&corsa_path);
    assert_eq!(
        changed.status,
        Some(1),
        "{}\n{}",
        changed.stdout,
        changed.stderr
    );
    assert_eq!(
        diagnostics_for(&parse_json(&changed), "src/Profile.vue"),
        vec!["error:4:22 [TS2322] Type 'string' is not assignable to type 'number'."],
        "a recreated dependency must be checked with its new content:\n{}",
        changed.stdout
    );

    project.write("src/types.ts", TYPES);
    let restored = project.check(&corsa_path);
    assert_eq!(
        restored.status,
        Some(0),
        "{}\n{}",
        restored.stdout,
        restored.stderr
    );
}

/// Corsa runs with `--incremental`, so an unchanged rerun reuses recorded
/// diagnostics; they must equal a fresh check, and a dependency-only edit must
/// still recheck the untouched importer.
#[test]
fn unchanged_reruns_keep_findings_and_dependency_edits_recheck_importers() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project = ProjectCase::new("batch-rerun");
    project.write("src/types.ts", TYPES_CHANGED);
    project.write("src/Profile.vue", PROFILE);
    let expected = vec!["error:4:22 [TS2322] Type 'string' is not assignable to type 'number'."];

    for run in 0..3 {
        let result = project.check(&corsa_path);
        assert_eq!(
            result.status,
            Some(1),
            "run {run}\n{}\n{}",
            result.stdout,
            result.stderr
        );
        assert_eq!(
            diagnostics_for(&parse_json(&result), "src/Profile.vue"),
            expected,
            "run {run}:\n{}",
            result.stdout
        );
    }

    project.write("src/types.ts", TYPES);
    let fixed = project.check(&corsa_path);
    assert_eq!(fixed.status, Some(0), "{}\n{}", fixed.stdout, fixed.stderr);

    project.write("src/types.ts", TYPES_CHANGED);
    let broken = project.check(&corsa_path);
    assert_eq!(
        diagnostics_for(&parse_json(&broken), "src/Profile.vue"),
        expected,
        "{}",
        broken.stdout
    );
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
    fn new(name: &str) -> Self {
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
            r#"{
  "compilerOptions": { "strict": true, "target": "ES2022", "module": "ESNext", "moduleResolution": "bundler", "noEmit": true },
  "include": ["src/**/*"]
}"#,
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

    fn rename(&self, from: &str, to: &str) {
        std::fs::rename(self.root.join(from), self.root.join(to)).unwrap();
    }

    fn remove(&self, path: &str) {
        std::fs::remove_file(self.root.join(path)).unwrap();
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

fn assert_program_files(json: &serde_json::Value, expected_sources: &[&str]) {
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
    assert_eq!(files, expected_sources, "program source files: {files:#?}");
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
