#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
//! Repeated `vize check` runs must follow tsconfig and package manifest edits
//! without stale program members or diagnostics (#3984: "tsconfig/package
//! changes invalidate the correct graph").
use crate::support::corsa_path;
use crate::support::corsa_requirement;
use crate::support::vue_stub;

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use vize_l0::cstr;

const BASE_TSCONFIG: &str = r#"{
  "compilerOptions": { "strict": true, "target": "ES2022", "module": "ESNext", "moduleResolution": "bundler", "noEmit": true },
  "include": ["src/**/*"]
}"#;

const EXCLUDING_TSCONFIG: &str = r#"{
  "compilerOptions": { "strict": true, "target": "ES2022", "module": "ESNext", "moduleResolution": "bundler", "noEmit": true },
  "include": ["src/**/*"],
  "exclude": ["src/legacy"]
}"#;

const APP: &str = r#"<script setup lang="ts">
const title: string = 'Vize'
</script>

<template>
  <h1>{{ title }}</h1>
</template>
"#;

const BROKEN_LEGACY: &str = "export const retries: number = 'three'\n";

const ALIASED_APP: &str = r#"<script setup lang="ts">
import { version } from '@/version'

const label: string = version
</script>

<template>
  <p>{{ label }}</p>
</template>
"#;

const PACKAGE_APP: &str = r#"<script setup lang="ts">
import { value } from 'local-pkg'

const label: string = value
</script>

<template>
  <p>{{ label }}</p>
</template>
"#;

#[test]
fn tsconfig_exclude_edits_add_and_remove_program_members() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project = ProjectCase::new("config-exclude");
    project.write("tsconfig.json", BASE_TSCONFIG);
    project.write("src/App.vue", APP);
    project.write("src/legacy/retries.ts", BROKEN_LEGACY);

    let included = project.check(&corsa_path);
    assert_eq!(
        included.status,
        Some(1),
        "{}\n{}",
        included.stdout,
        included.stderr
    );
    let json = parse_json(&included);
    assert_program_files(&json, &["src/App.vue", "src/legacy/retries.ts"]);
    assert_eq!(
        diagnostics_for(&json, "src/legacy/retries.ts"),
        vec!["error:1:14 [TS2322] Type 'string' is not assignable to type 'number'."],
        "{}",
        included.stdout
    );

    project.write("tsconfig.json", EXCLUDING_TSCONFIG);
    let excluded = project.check(&corsa_path);
    assert_eq!(
        excluded.status,
        Some(0),
        "{}\n{}",
        excluded.stdout,
        excluded.stderr
    );
    assert_program_files(&parse_json(&excluded), &["src/App.vue"]);

    project.write("tsconfig.json", BASE_TSCONFIG);
    let restored = project.check(&corsa_path);
    assert_eq!(
        restored.status,
        Some(1),
        "{}\n{}",
        restored.stdout,
        restored.stderr
    );
    assert_eq!(
        diagnostics_for(&parse_json(&restored), "src/legacy/retries.ts"),
        diagnostics_for(&json, "src/legacy/retries.ts"),
        "{}",
        restored.stdout
    );
}

#[test]
fn tsconfig_paths_edits_retarget_aliased_imports() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project = ProjectCase::new("config-paths");
    project.write("tsconfig.json", &paths_tsconfig("src/v1/*"));
    project.write(
        "src/v1/version.ts",
        "export const version: string = '1.0.0'\n",
    );
    project.write("src/v2/version.ts", "export const version: number = 2\n");
    project.write("src/App.vue", ALIASED_APP);

    let clean = project.check(&corsa_path);
    assert_eq!(clean.status, Some(0), "{}\n{}", clean.stdout, clean.stderr);

    project.write("tsconfig.json", &paths_tsconfig("src/v2/*"));
    let retargeted = project.check(&corsa_path);
    assert_eq!(
        retargeted.status,
        Some(1),
        "{}\n{}",
        retargeted.stdout,
        retargeted.stderr
    );
    assert_eq!(
        diagnostics_for(&parse_json(&retargeted), "src/App.vue"),
        vec!["error:4:7 [TS2322] Type 'number' is not assignable to type 'string'."],
        "{}",
        retargeted.stdout
    );

    project.write("tsconfig.json", &paths_tsconfig("src/v1/*"));
    let restored = project.check(&corsa_path);
    assert_eq!(
        restored.status,
        Some(0),
        "{}\n{}",
        restored.stdout,
        restored.stderr
    );
}

#[test]
fn package_exports_edits_retarget_package_types() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project = ProjectCase::new("config-package-exports");
    project.write("tsconfig.json", BASE_TSCONFIG);
    project.write(
        "node_modules/local-pkg/string.d.ts",
        "export declare const value: string\n",
    );
    project.write(
        "node_modules/local-pkg/number.d.ts",
        "export declare const value: number\n",
    );
    project.write(
        "node_modules/local-pkg/package.json",
        &package_manifest("./string.d.ts"),
    );
    project.write("src/App.vue", PACKAGE_APP);

    let clean = project.check(&corsa_path);
    assert_eq!(clean.status, Some(0), "{}\n{}", clean.stdout, clean.stderr);

    project.write(
        "node_modules/local-pkg/package.json",
        &package_manifest("./number.d.ts"),
    );
    let retargeted = project.check(&corsa_path);
    assert_eq!(
        retargeted.status,
        Some(1),
        "{}\n{}",
        retargeted.stdout,
        retargeted.stderr
    );
    assert_eq!(
        diagnostics_for(&parse_json(&retargeted), "src/App.vue"),
        vec!["error:4:7 [TS2322] Type 'number' is not assignable to type 'string'."],
        "{}",
        retargeted.stdout
    );

    project.write(
        "node_modules/local-pkg/package.json",
        &package_manifest("./string.d.ts"),
    );
    let restored = project.check(&corsa_path);
    assert_eq!(
        restored.status,
        Some(0),
        "{}\n{}",
        restored.stdout,
        restored.stderr
    );
}

fn paths_tsconfig(target: &str) -> String {
    cstr!(
        r#"{{
  "compilerOptions": {{ "strict": true, "target": "ES2022", "module": "ESNext", "moduleResolution": "bundler", "noEmit": true, "paths": {{ "@/*": ["./{target}"] }} }},
  "include": ["src/**/*"]
}}"#
    )
    .as_str()
    .to_owned()
}

fn package_manifest(types: &str) -> String {
    cstr!(
        r#"{{ "name": "local-pkg", "version": "1.0.0", "exports": {{ ".": {{ "types": "{types}" }} }} }}"#
    )
    .as_str()
    .to_owned()
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
        Self { root }
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
