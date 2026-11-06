//! A generic SFC with emits and declared slots remains castable to Vue's
//! public `Component` union (#11966).

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};

const PUBLIC_CONTEXT_COMPONENT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/",
    "generic-component-public-context-11966/Generic.vue.txt"
));
const PUBLIC_CONTEXT_CONSUMER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/",
    "generic-component-public-context-11966/consumer.ts.txt"
));
const PUBLIC_CONTEXT_TSCONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/",
    "generic-component-public-context-11966/tsconfig.json.txt"
));

#[test]
fn generic_component_with_emits_and_slots_casts_to_component() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project = super::real_vue::create_project_case(
        "generic-component-cast-11966",
        &[
            (
                "src/List.vue",
                r#"<script setup lang="ts" generic="R extends { id: string }">
defineProps<{ rows: R[] }>();
defineEmits<{ visible: [ids: string[]] }>();
defineSlots<{ cell(props: { row: R }): unknown }>();
</script>

<template>
  <ul><li v-for="row in rows" :key="row.id"><slot name="cell" :row="row" /></li></ul>
</template>
"#,
            ),
            (
                "src/use.ts",
                r#"import type { Component } from 'vue';
import List from './List.vue';

export const asComponent = List as Component;
"#,
            ),
        ],
    );
    let actual = snapshot_project_diagnostics(&project);
    let _ = std::fs::remove_dir_all(project);
    assert_eq!(
        actual,
        Some(Vec::new()),
        "the reported cast must match vue-tsc 3.3.11",
    );
}

#[test]
fn generic_component_helpers_keep_precise_emits_slots_and_component_overlap() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project = super::real_vue::create_project_case(
        "generic-component-public-context-11966",
        &[
            ("src/Generic.vue", PUBLIC_CONTEXT_COMPONENT),
            ("src/consumer.ts", PUBLIC_CONTEXT_CONSUMER),
            ("tsconfig.json", PUBLIC_CONTEXT_TSCONFIG),
        ],
    );
    let actual = snapshot_project_diagnostics(&project);
    let _ = std::fs::remove_dir_all(project);
    assert_eq!(
        actual,
        Some(Vec::new()),
        "public helper extraction and Vue Component overlap must both remain precise",
    );
}

#[test]
fn generic_component_without_emits_keeps_vue_component_cast_error() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project = super::real_vue::create_project_case(
        "generic-component-cast-without-emits-11966",
        &[
            (
                "src/List.vue",
                r#"<script setup lang="ts" generic="R extends { id: string }">
defineProps<{ rows: R[] }>();
defineSlots<{ cell(props: { row: R }): unknown }>();
</script>

<template><slot name="cell" :row="rows[0]!" /></template>
"#,
            ),
            (
                "src/use.ts",
                r#"import type { Component } from 'vue';
import List from './List.vue';
export const asComponent = List as Component;
"#,
            ),
        ],
    );
    let actual = snapshot_project_diagnostics(&project).expect("native diagnostics");
    let _ = std::fs::remove_dir_all(project);
    assert_eq!(actual.len(), 1, "{actual:#?}");
    assert_eq!(actual[0].0, "src/use.ts");
    assert_eq!(actual[0].1, Some(2352));
}
