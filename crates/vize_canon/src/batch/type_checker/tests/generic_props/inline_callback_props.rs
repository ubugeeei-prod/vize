//! Inline callback props on a generic child (#3446).
//!
//! A generic child's props come from its `__vizeCheck<T>(props)` call, so the
//! per-prop alias resolves to `unknown` and an inline arrow annotated with it
//! has no contextual type at all. Under `strict` that produced a `TS7006` on a
//! parameter vue-tsc infers from the child's generic — a new error on correct
//! code — while the real check still ran in the checker call.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};
use vize_carton::String;

/// vue-tsc 3.3.4 on this workspace:
///
/// ```text
/// src/Parent.vue(6,48): error TS2322: Type 'number' is not assignable to type 'string'.
/// src/Parent.vue(7,21): error TS2322: Type 'string' is not assignable to type 'number'.
/// ```
///
/// Line 6 column 39 is the `item` parameter of the inline arrow, which vize
/// used to flag `TS7006` and vue-tsc does not flag at all. Line 7 column 21 is
/// the `id` key of the offending object literal, which vize used to anchor one
/// byte to the right.
///
/// The line 6 assignability error itself is still anchored differently: vize
/// reports the whole signature at the `pick` attribute name (column 32) where
/// vue-tsc reports the leaf inside the arrow body (column 48). That is the one
/// part of #3446 this does not close, and it needs the generic child's prop
/// type to be resolved per prop rather than only inside the checker call.
#[test]
fn inline_callback_prop_on_a_generic_child_is_contextually_typed() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "generic-inline-callback-prop",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts" generic="T extends { id: number }">
defineProps<{ items: T[]; pick: (item: T) => string }>()
</script>

<template><span /></template>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
</script>

<template>
  <Child :items="[{ id: 1 }]" :pick="(item) => item.id" />
  <Child :items="[{ id: 'x' }]" :pick="() => 'y'" />
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
                String::from("src/Parent.vue"),
                Some(2322),
                String::from(
                    "6:32:error Type '(item: { id: number; }) => number' is not assignable to type '(item: { id: number; }) => string'.\n\
                     Type 'number' is not assignable to type 'string'."
                ),
            ),
            (
                String::from("src/Parent.vue"),
                Some(2322),
                String::from("7:21:error Type 'string' is not assignable to type 'number'."),
            ),
        ]
    );
}
