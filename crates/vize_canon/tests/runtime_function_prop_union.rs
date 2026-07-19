use std::path::Path;

use vize_canon::{BatchTypeChecker, BatchTypeCheckerTrait, SfcTypeCheckOptions, type_check_sfc};

const COMPONENT: &str = r#"<script setup lang="ts">
const props = defineProps({
  data: { type: [Array, Function], required: true },
  labelOrPredicate: { type: [String, Function], required: true },
  formatter: { type: Function, required: true },
})
void props
</script>

<template><div /></template>
"#;

#[test]
fn runtime_function_union_members_generate_parseable_typescript() {
    let virtual_ts = type_check_sfc(
        COMPONENT,
        &SfcTypeCheckOptions::new("FlexibleData.vue").with_virtual_ts(),
    )
    .virtual_ts
    .expect("virtual TypeScript should be generated");

    assert!(
        virtual_ts.contains("data: unknown[] | ((...args: any[]) => any);"),
        "the Crater-style Array/Function prop must parenthesize its function member:\n{virtual_ts}"
    );
    assert!(
        virtual_ts.contains("labelOrPredicate: string | ((...args: any[]) => any);"),
        "a scalar/function union must preserve the same precedence:\n{virtual_ts}"
    );
    assert!(
        virtual_ts.contains("formatter: (...args: any[]) => any;"),
        "a standalone Function constructor must retain its callable shape:\n{virtual_ts}"
    );
    assert!(
        !virtual_ts.contains("unknown[] | (...args: any[]) => any"),
        "an unparenthesized function union must never be emitted:\n{virtual_ts}"
    );

    let allocator = oxc_allocator::Allocator::default();
    let parsed =
        oxc_parser::Parser::new(&allocator, virtual_ts.as_str(), oxc_span::SourceType::ts())
            .parse();
    assert!(
        !parsed.panicked && parsed.errors.is_empty(),
        "runtime function unions must produce parseable TypeScript: {:#?}\n{virtual_ts}",
        parsed.errors
    );
}

#[test]
fn runtime_function_unions_accept_both_members_and_reject_other_types() {
    let project = tempfile::tempdir().expect("temporary project should be created");
    write_project(project.path());

    let mut checker = BatchTypeChecker::new(project.path()).expect("type checker should start");
    checker.scan_project().expect("project should scan");
    let diagnostics = checker
        .check_project()
        .expect("project should type check")
        .diagnostics;

    let unexpected: Vec<_> = diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.code == Some(1385)
                || (diagnostic.file.ends_with("consumer.ts") && diagnostic.code != Some(2322))
        })
        .collect();
    assert!(
        unexpected.is_empty(),
        "valid arrays, strings, and functions must remain diagnostic-free: {unexpected:#?}"
    );

    let rejected_assignments = diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.file.ends_with("consumer.ts") && diagnostic.code == Some(2322)
        })
        .count();
    assert_eq!(
        rejected_assignments, 3,
        "each value outside the declared runtime prop types must be rejected: {diagnostics:#?}"
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
"#,
    );
    write_file(root, "src/FlexibleData.vue", COMPONENT);
    write_file(
        root,
        "src/consumer.ts",
        r#"import type { Props } from "./FlexibleData.vue";

const acceptedArray: Props = {
  data: [],
  labelOrPredicate: "ready",
  formatter: () => "done",
};
const acceptedFunctions: Props = {
  data: () => ({ id: 1 }),
  labelOrPredicate: (label: string) => label.length > 0,
  formatter: (value: unknown) => String(value),
};

const rejectedData: Props = {
  data: "not-an-array-or-function",
  labelOrPredicate: "ready",
  formatter: () => "done",
};
const rejectedPredicate: Props = {
  data: [],
  labelOrPredicate: 42,
  formatter: () => "done",
};
const rejectedFormatter: Props = {
  data: [],
  labelOrPredicate: "ready",
  formatter: "not-a-function",
};

void acceptedArray;
void acceptedFunctions;
void rejectedData;
void rejectedPredicate;
void rejectedFormatter;
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
