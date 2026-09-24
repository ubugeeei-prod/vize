#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::json;

mod content_mapper_declaration_map_support;
use content_mapper_declaration_map_support::assert_authored_vue_declaration_map;

const TSGO_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_TSGO";
const VUE_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_VUE";
const PACKAGE_APP_SOURCE: &str = r#"<script setup lang="ts">
import { Counter } from "@fixture/content-ui";

defineProps<{ initial: number }>();
</script>

<template>
  <Counter :count="initial" />
</template>
"#;
const COUNTER_SOURCE: &str = r#"<script setup lang="ts">
defineProps<{ count: number }>();
</script>

<template>
  <button>{{ count.toFixed(0) }}</button>
</template>
"#;

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn copy_fixture(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        if entry.file_name() == "node_modules" {
            continue;
        }
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_fixture(&source_path, &destination_path);
        } else {
            std::fs::copy(source_path, destination_path).unwrap();
        }
    }
}

fn install_packages(project_root: &Path) {
    let mapper_root = project_root.join("node_modules/vize");
    std::fs::create_dir_all(&mapper_root).unwrap();
    std::fs::write(
        mapper_root.join("package.json"),
        serde_json::to_vec_pretty(&json!({
            "name": "vize",
            "private": true,
            "typescript": {
                "contentMapper": {
                    "exec": [env!("CARGO_BIN_EXE_vize"), "content-mapper"],
                    "compilerOptions": ["noUnusedLocals"],
                },
            },
        }))
        .unwrap(),
    )
    .unwrap();

    let vue_source = std::env::var_os(VUE_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let store = workspace_root().join("node_modules/.pnpm");
            let mut candidates = std::fs::read_dir(&store)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", store.display()))
                .filter_map(Result::ok)
                .filter(|entry| entry.file_name().to_string_lossy().starts_with("vue@3."))
                .map(|entry| entry.path().join("node_modules/vue"))
                .filter(|path| path.join("package.json").is_file())
                .collect::<Vec<_>>();
            candidates.sort();
            candidates.pop().unwrap_or_else(|| {
                panic!(
                    "no Vue 3 package found under {}; set {VUE_ENV}",
                    store.display()
                )
            })
        });
    assert!(vue_source.join("package.json").is_file());
    symlink_dir(&vue_source, &project_root.join("node_modules/vue"));
}

fn expose_ui_package(project_root: &Path) {
    let ui_root = project_root.join("references/ui");
    std::fs::write(
        ui_root.join("package.json"),
        r#"{
  "name": "@fixture/content-ui",
  "private": true,
  "type": "module",
  "types": "./dist/index.d.ts",
  "exports": {
    ".": {
      "types": "./dist/index.d.ts",
      "default": "./dist/index.js"
    }
  }
}"#,
    )
    .unwrap();
    symlink_dir(
        &ui_root,
        &project_root.join("references/app/node_modules/@fixture/content-ui"),
    );
    let app = project_root.join("references/app/src/App.vue");
    let source = std::fs::read_to_string(&app).unwrap();
    let source = source.replace("../../ui/src/index", "@fixture/content-ui");
    assert_eq!(source, PACKAGE_APP_SOURCE);
    std::fs::write(app, source).unwrap();
}

fn symlink_dir(source: &Path, target: &Path) {
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(source, target).unwrap();
}

fn run_build(tsgo: &Path, project_root: &Path) -> Output {
    Command::new(tsgo)
        .current_dir(project_root)
        .args([
            "--build",
            "references/tsconfig.json",
            "--runExternalCode",
            "--pretty",
            "false",
            "--verbose",
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to run {}: {error}", tsgo.display()))
}

fn output_text(output: &Output) -> String {
    format!(
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    )
}

fn diagnostic_lines(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter(|line| line.split_once(": error TS").is_some())
        .collect()
}

#[test]
fn standard_tsgo_builds_package_exported_vue_project_references() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!(
            "skipping package project-reference Content Mapper oracle: {TSGO_ENV} is not set"
        );
        return;
    };
    assert!(
        tsgo.is_file(),
        "{TSGO_ENV} is not a file: {}",
        tsgo.display()
    );

    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/content_mapper_project");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-package-references-")
        .tempdir_in(cases_root)
        .unwrap();
    copy_fixture(&fixture, project.path());
    install_packages(project.path());
    expose_ui_package(project.path());

    let initial = run_build(&tsgo, project.path());
    assert!(initial.status.success(), "{}", output_text(&initial));
    for declaration in [
        "references/ui/dist/Counter.d.vue.ts",
        "references/ui/dist/index.d.ts",
        "references/app/dist/App.d.vue.ts",
        "references/app/dist/main.d.ts",
    ] {
        assert!(
            project.path().join(declaration).is_file(),
            "missing build output {declaration}"
        );
    }
    let counter_source = assert_authored_vue_declaration_map(
        project.path(),
        "references/ui/dist/Counter.d.vue.ts",
        "Counter",
    );
    assert_eq!(counter_source.content, COUNTER_SOURCE);
    let app_source = assert_authored_vue_declaration_map(
        project.path(),
        "references/app/dist/App.d.vue.ts",
        "App",
    );
    assert_eq!(app_source.content, PACKAGE_APP_SOURCE);

    let app = project.path().join("references/app/src/App.vue");
    let valid_app = std::fs::read_to_string(&app).unwrap();
    assert_eq!(valid_app, PACKAGE_APP_SOURCE);
    let invalid_app = valid_app.replace(":count=\"initial\"", r#":count="String(initial)""#);
    assert_ne!(valid_app, invalid_app);
    std::fs::write(&app, invalid_app).unwrap();

    let broken = run_build(&tsgo, project.path());
    assert!(
        !broken.status.success(),
        "broken package reference build passed unexpectedly"
    );
    let broken_text = output_text(&broken);
    assert_eq!(
        diagnostic_lines(&broken_text),
        vec![
            "references/app/src/App.vue(8,13): error TS2322: Type 'string' is not assignable to type 'number'.",
            "references/app/src/App.vue(8,35): error TS2322: Type 'string' is not assignable to type 'number'.",
        ]
    );

    std::fs::write(&app, valid_app).unwrap();
    let repaired = run_build(&tsgo, project.path());
    assert!(repaired.status.success(), "{}", output_text(&repaired));
}
