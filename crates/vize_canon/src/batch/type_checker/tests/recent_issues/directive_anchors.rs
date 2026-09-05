//! Where a component-binding diagnostic anchors (#3462).
//!
//! vue-tsc assigns every binding to the child's corresponding prop, so a
//! wrongly-typed one is a `TS2322` at the attribute name. vize used to anchor
//! an `@event` handler and an argument-less `v-model` at the bound expression
//! instead — the handler additionally as a `TS2345`, because it was passed as a
//! call argument rather than assigned. Both oracles below are vue-tsc 3.3.4 on
//! byte-identical workspaces.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};
use vize_s0::String;

/// vue-tsc 3.3.4:
///
/// ```text
/// src/Parent.vue(7,11): error TS2322: Type '(value: string) => string' is not assignable to type '(value: number) => any'.
/// src/Parent.vue(7,24): error TS2322: Type '(value: string) => string' is not assignable to type '() => any'.
/// src/Parent.vue(8,29): error TS2345: Argument of type 'number' is not assignable to parameter of type 'string'.
/// ```
///
/// Column 11 is the `pick` of `@pick` and column 24 the `done` of `@done`;
/// before the fix vize reported `TS2345` at columns 17 and 30, the `take`
/// expressions. Line 8 — an error *inside* an inline arrow — was already
/// byte-identical and must stay on the authored bytes.
///
/// The pinned TypeScript-Go build normalizes vize's named rest tuple
/// `(...args: [value: number]) => any` to `(value: number) => any` when it
/// renders the diagnostic. The parameter label and listener shape therefore
/// stay byte-identical to vue-tsc instead of creating a message-only
/// divergence (#3447).
#[test]
fn component_event_handler_anchors_at_the_event_name() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "component-event-handler-anchor",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineEmits<{ pick: [value: number]; done: [] }>()
</script>

<template><span /></template>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
const take = (value: string) => value
</script>

<template>
  <Child @pick="take" @done="take" />
  <Child @pick="(v) => take(v)" />
</template>
"#,
            ),
        ],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };
    let snapshot: Vec<_> = snapshot
        .into_iter()
        .map(|(path, code, message)| {
            let message = message
                .replace(
                    "Types of parameters 'value' and 'value' are incompatible.",
                    "Types of parameters 'value' and '<target>' are incompatible.",
                )
                .replace(
                    "Types of parameters 'value' and 'args' are incompatible.",
                    "Types of parameters 'value' and '<target>' are incompatible.",
                );
            (path, code, String::from(message.as_str()))
        })
        .collect();

    assert_eq!(
        snapshot,
        vec![
            (
                String::from("src/Parent.vue"),
                Some(2322),
                String::from(
                    "7:11:error Type '(value: string) => string' is not assignable to type '(value: number) => any'.\n\
                     Types of parameters 'value' and '<target>' are incompatible.\n\
                     Type 'number' is not assignable to type 'string'."
                ),
            ),
            (
                String::from("src/Parent.vue"),
                Some(2322),
                String::from(
                    "7:24:error Type '(value: string) => string' is not assignable to type '() => any'.\n\
                     Target signature provides too few arguments. Expected 1 or more, but got 0."
                ),
            ),
            (
                String::from("src/Parent.vue"),
                Some(2345),
                String::from(
                    "8:29:error Argument of type 'number' is not assignable to parameter of type 'string'."
                ),
            ),
        ]
    );
}

/// vue-tsc 3.3.4:
///
/// ```text
/// src/ModelParent.vue(10,10): error TS2322: Type 'string' is not assignable to type 'number'.
/// src/ModelParent.vue(10,33): error TS2322: Type 'number' is not assignable to type 'string'.
/// ```
///
/// Column 10 is the `v` of `v-model` and column 33 the `title` of
/// `v-model:title`. The named form already agreed; the argument-less one binds
/// `modelValue`, a name that appears nowhere in the source, so before the fix
/// it fell back to the bound expression at column 19.
#[test]
fn argument_less_v_model_anchors_at_the_directive() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "v-model-directive-anchor",
        &[
            (
                "src/ModelChild.vue",
                r#"<script setup lang="ts">
defineModel<number>()
defineModel<string>('title')
</script>

<template><span /></template>
"#,
            ),
            (
                "src/ModelParent.vue",
                r#"<script setup lang="ts">
import { ref } from 'vue'
import Child from './ModelChild.vue'

const text = ref('hello')
const num = ref(1)
</script>

<template>
  <Child v-model="text" v-model:title="num" />
</template>
"#,
            ),
        ],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };

    assert_eq!(
        snapshot,
        vec![
            (
                String::from("src/ModelParent.vue"),
                Some(2322),
                String::from("10:10:error Type 'string' is not assignable to type 'number'."),
            ),
            (
                String::from("src/ModelParent.vue"),
                Some(2322),
                String::from("10:33:error Type 'number' is not assignable to type 'string'."),
            ),
        ]
    );
}
