use std::path::Path;

use vize_canon::{BatchTypeChecker, BatchTypeCheckerTrait, SfcTypeCheckOptions, type_check_sfc};
use vize_carton::{String, ToCompactString};

#[test]
fn unresolved_component_events_accept_multi_argument_handlers() {
    let source = r#"<script setup lang="ts">
import UnknownChild from './UnknownChild.vue'
function handleSelect(key: string, path: string[]) {
  void key
  void path
}
</script>
<template><UnknownChild @select="handleSelect" /></template>
"#;
    let virtual_ts = type_check_sfc(
        source,
        &SfcTypeCheckOptions::new("App.vue").with_virtual_ts(),
    )
    .virtual_ts
    .expect("virtual TypeScript should be generated");
    assert!(
        virtual_ts.contains("? ((...args: any[]) => unknown) :"),
        "an unresolved component emit must preserve unknown arity:\n{virtual_ts}"
    );

    let project = create_project(&[
        ("src/App.vue", source),
        (
            "src/UnknownChild.vue",
            "<script setup lang=\"ts\"></script>\n<template><div /></template>\n",
        ),
    ]);
    assert_no_diagnostic(
        project.path(),
        "App.vue",
        2345,
        "an unresolved component emit must not reject a valid multi-argument handler",
    );
}

#[test]
fn resolved_component_events_still_check_every_argument() {
    let parent = r#"<script setup lang="ts">
import KnownChild from './KnownChild.vue'
function handleSelect(key: number, path: string[]) {
  void key
  void path
}
</script>
<template><KnownChild @select="handleSelect" /></template>
"#;
    let child = r#"<script setup lang="ts">
defineEmits<{ select: [key: string, path: string[]] }>()
</script>
<template><div /></template>
"#;
    let project = create_project(&[("src/App.vue", parent), ("src/KnownChild.vue", child)]);
    assert_has_diagnostic(
        project.path(),
        "App.vue",
        2345,
        "a resolved component emit must reject a handler with the wrong first argument type",
    );
}

#[test]
fn resolved_component_events_accept_the_exact_handler_tuple() {
    let parent = r#"<script setup lang="ts">
import KnownChild from './KnownChild.vue'
function handleSelect(key: string, path: string[]) {
  void key
  void path
}
</script>
<template><KnownChild @select="handleSelect" /></template>
"#;
    let child = r#"<script setup lang="ts">
defineEmits<{ select: [key: string, path: string[]] }>()
</script>
<template><div /></template>
"#;
    let project = create_project(&[("src/App.vue", parent), ("src/KnownChild.vue", child)]);
    assert_no_diagnostic(
        project.path(),
        "App.vue",
        2345,
        "a resolved component emit must accept its exact handler tuple",
    );
}

#[test]
fn native_events_still_reject_extra_required_arguments() {
    let source = r#"<script setup lang="ts">
function handleClick(event: PointerEvent, required: string) {
  void event
  void required
}
</script>
<template><button @click="handleClick">Click</button></template>
"#;
    let project = create_project(&[("src/App.vue", source)]);
    assert_has_diagnostic(
        project.path(),
        "App.vue",
        2345,
        "a native event must retain its single-event listener contract",
    );
}

fn create_project(files: &[(&str, &str)]) -> tempfile::TempDir {
    let project = tempfile::tempdir().expect("temporary project should be created");
    write_file(
        project.path(),
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    );
    write_file(
        project.path(),
        "node_modules/vue/package.json",
        r#"{ "name": "vue", "types": "index.d.ts" }"#,
    );
    write_file(
        project.path(),
        "node_modules/vue/index.d.ts",
        r#"export interface ComponentPublicInstance {
  $attrs: Record<string, unknown>;
  $slots: Record<string, unknown>;
  $refs: Record<string, unknown>;
  $emit: (...args: unknown[]) => void;
}
"#,
    );
    for (path, source) in files {
        write_file(project.path(), path, source);
    }
    project
}

fn project_diagnostics(root: &Path) -> Vec<(String, Option<u32>, String)> {
    let mut checker = BatchTypeChecker::new(root).expect("type checker should start");
    checker.scan_project().expect("project should scan");
    checker
        .check_project()
        .expect("project should type check")
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            (
                diagnostic.file.display().to_compact_string(),
                diagnostic.code,
                diagnostic.message,
            )
        })
        .collect()
}

fn assert_no_diagnostic(root: &Path, file: &str, code: u32, reason: &str) {
    let diagnostics = project_diagnostics(root);
    assert!(
        !diagnostics
            .iter()
            .any(|(path, actual, _)| path.ends_with(file) && *actual == Some(code)),
        "{reason}: {diagnostics:#?}"
    );
}

fn assert_has_diagnostic(root: &Path, file: &str, code: u32, reason: &str) {
    let diagnostics = project_diagnostics(root);
    assert!(
        diagnostics
            .iter()
            .any(|(path, actual, _)| path.ends_with(file) && *actual == Some(code)),
        "{reason}: {diagnostics:#?}"
    );
}

fn write_file(root: &Path, path: &str, source: &str) {
    let path = root.join(path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("parent directory should be created");
    }
    std::fs::write(path, source).expect("fixture should be written");
}
