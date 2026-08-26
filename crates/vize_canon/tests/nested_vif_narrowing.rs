use std::path::Path;

use vize_canon::{BatchTypeChecker, BatchTypeCheckerTrait};
use vize_s0::cstr;

#[test]
fn nested_else_v_for_keeps_the_outer_discriminant_narrowing() {
    let project = create_project();
    let source = cstr!(
        "{NESTED_VIF_SFC}\n<style>\n{}</style>\n",
        ".pad {}\n".repeat(400)
    );
    write_project_file(project.path(), "src/App.vue", source.as_str());

    let mut checker = BatchTypeChecker::new(project.path()).expect("Corsa should be available");
    checker.scan_project().expect("project should scan");
    let result = checker.check_project().expect("project should type check");
    let discriminant_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == Some(2367))
        .collect();

    assert!(
        discriminant_errors.is_empty(),
        "the nested v-for must not compare an already narrowed discriminant: {discriminant_errors:#?}"
    );
}

#[test]
fn v_for_callback_rechecks_the_outer_discriminant() {
    let project = create_project();
    write_project_file(
        project.path(),
        "src/Child.vue",
        r#"<script setup lang="ts">
defineProps<{ align: "left" | "right" }>();
</script>
<template><div /></template>
"#,
    );
    write_project_file(
        project.path(),
        "src/App.vue",
        r#"<script setup lang="ts">
import Child from "./Child.vue";

type Item =
  | { type: "text"; text: string }
  | { type: "container"; children: string[]; align: "left" | "right" };

const item = {} as Item;
</script>

<template>
  <div v-if="item.type === 'container'">
    <Child
      v-for="child in item.children"
      :key="child"
      :align="item.align"
    />
  </div>
</template>
"#,
    );

    let mut checker = BatchTypeChecker::new(project.path()).expect("Corsa should be available");
    checker.scan_project().expect("project should scan");
    let result = checker.check_project().expect("project should type check");
    let property_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == Some(2339))
        .collect();

    assert!(
        property_errors.is_empty(),
        "the v-for callback must restore the outer discriminant narrowing: {property_errors:#?}"
    );
}

fn create_project() -> tempfile::TempDir {
    let project = tempfile::tempdir().expect("temp project should be created");
    write_project_file(
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
    write_project_file(
        project.path(),
        "node_modules/vue/package.json",
        r#"{ "name": "vue", "types": "index.d.ts" }"#,
    );
    write_project_file(
        project.path(),
        "node_modules/vue/index.d.ts",
        r#"export interface Ref<T = unknown> { value: T }
export function ref<T>(value: T): Ref<T>;
export function computed<T>(getter: () => T): Readonly<Ref<T>>;
"#,
    );
    project
}

fn write_project_file(project_root: &Path, path: &str, source: &str) {
    let path = project_root.join(path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, source).unwrap();
}

const NESTED_VIF_SFC: &str = r#"<script setup lang="ts">
import { computed, ref } from "vue";
interface BarItem { type: "hunk-bar"; lines: number }
interface LineItem { type: "line"; text: string }
type ViewItem = BarItem | { type: "section"; lines: LineItem[] }
const rows = ref<(LineItem | BarItem)[]>([]);
const items = computed<ViewItem[]>(() => []);
</script>

<template>
  <template v-for="(item, i) in items" :key="i">
    <button v-if="item.type === 'hunk-bar'">{{ item.lines }}</button>
    <div v-else>
      <div v-for="(row, j) in item.lines" :key="j">{{ row.text }}</div>
    </div>
  </template>
</template>"#;
