use std::path::Path;

use vize_canon::{BatchTypeChecker, BatchTypeCheckerTrait, SfcTypeCheckOptions, type_check_sfc};

#[test]
fn imported_ref_is_unwrapped_in_template_scope() {
    let result = type_check_sfc(
        APP_SFC,
        &SfcTypeCheckOptions::new("App.vue").with_virtual_ts(),
    );
    let virtual_ts = result.virtual_ts.expect("virtual ts should be generated");

    assert!(
        virtual_ts.contains("type __R_sharedRef = typeof sharedRef;"),
        "imported refs need a pre-template type capture:\n{virtual_ts}"
    );
    assert!(
        virtual_ts.contains("var sharedRef: __U<__R_sharedRef> = undefined as any;"),
        "imported refs need an unwrapped template shadow:\n{virtual_ts}"
    );
    assert!(
        virtual_ts.contains("var format: __U<__R_format> = undefined as any;"),
        "non-ref named imports should keep their type through the same safe helper:\n{virtual_ts}"
    );
    assert!(
        !virtual_ts.contains("type __R_BooleanProp"),
        "default component imports should not be shadowed:\n{virtual_ts}"
    );
}

#[test]
fn imported_ref_satisfies_primitive_component_prop() {
    let project = tempfile::tempdir().expect("temp project should be created");
    write_project(project.path());

    let mut checker = BatchTypeChecker::new(project.path()).expect("checker should start");
    checker.scan_project().expect("project should scan");
    let result = checker.check_project().expect("project should type check");
    let relevant: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.file.ends_with("App.vue") && matches!(diagnostic.code, Some(2322 | 2345))
        })
        .collect();

    assert!(
        relevant.is_empty(),
        "an imported Ref<boolean> should satisfy a boolean prop: {relevant:#?}"
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
        r#"export interface Ref<T = unknown, S = T> { value: T }
export function ref<T>(value: T): Ref<T, T>;
export interface ComponentPublicInstance {
  $attrs: Record<string, unknown>;
  $slots: Record<string, unknown>;
  $refs: Record<string, unknown>;
  $emit: (...args: unknown[]) => void;
}
"#,
    );
    write_file(root, "src/App.vue", APP_SFC);
    write_file(
        root,
        "src/other.ts",
        "import { ref } from 'vue';\nexport const sharedRef = ref(false);\nexport function format(value: boolean): string { return String(value); }\n",
    );
    write_file(
        root,
        "src/BooleanProp.vue",
        r#"<script setup lang="ts">
defineProps<{ boolProp?: boolean }>()
</script>
<template><div /></template>
"#,
    );
}

fn write_file(root: &Path, path: &str, source: &str) {
    let path = root.join(path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("parent directory should be created");
    }
    std::fs::write(path, source).expect("fixture should be written");
}

const APP_SFC: &str = r#"<script setup lang="ts">
import { format, sharedRef } from "./other.ts";
import BooleanProp from "./BooleanProp.vue";
</script>

<template>
  <BooleanProp :boolProp="sharedRef" />
  <div>{{ format(sharedRef) }}</div>
</template>
"#;
