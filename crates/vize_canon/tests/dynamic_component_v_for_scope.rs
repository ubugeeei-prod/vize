use std::path::Path;

use vize_canon::{BatchTypeChecker, BatchTypeCheckerTrait};

#[test]
fn dynamic_component_is_before_v_for_sees_loop_alias() {
    let project = tempfile::tempdir().expect("temp project should be created");
    write_project(project.path());

    let Ok(mut checker) = BatchTypeChecker::new(project.path()) else {
        return;
    };
    checker.scan_project().expect("project should scan");
    let result = checker.check_project().expect("project should check");

    let relevant: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.file.ends_with("App.vue"))
        .filter(|diagnostic| {
            matches!(diagnostic.code, Some(2304 | 2339)) && diagnostic.message.contains("part")
        })
        .collect();

    assert!(
        relevant.is_empty(),
        "same-element v-for aliases must scope dynamic component is bindings: {relevant:#?}"
    );
}

fn write_project(root: &Path) {
    write_file(
        root,
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
        root,
        "node_modules/vue/package.json",
        r#"{ "name": "vue", "types": "index.d.ts" }"#,
    );
    write_file(
        root,
        "node_modules/vue/index.d.ts",
        r#"export interface ComponentPublicInstance {
  $attrs: Record<string, unknown>;
  $slots: Record<string, unknown>;
  $refs: Record<string, unknown>;
  $emit: (...args: unknown[]) => void;
}
export interface VNodeProps {}
export interface AllowedComponentProps {}
export interface ComponentCustomProps {}
export type NativeElements = Record<string, Record<string, unknown>>;
export type Directive<T = unknown, V = unknown> = (element: unknown, binding: { value: V }) => void;
"#,
    );
    write_file(root, "src/App.vue", APP_SFC);
}

fn write_file(root: &Path, path: &str, source: &str) {
    let path = root.join(path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("parent directory should be created");
    }
    std::fs::write(path, source).expect("fixture should be written");
}

const APP_SFC: &str = r#"<script setup lang="ts">
const parts = [{ tag: "span", value: "a" }];
</script>

<template>
  <p>
    <component :is="part.tag" v-for="part in parts" :key="part.value">{{ part.value }}</component>
  </p>
</template>
"#;
