//! The resolved-props binding under a narrowing `v-if`, and under
//! `noUnusedLocals` (#3446, #3590).
//!
//! `inline_callback_props` already pins the callback anchors in a guarded and a
//! looped scope, but leaves two properties of the resolution untested:
//!
//! * its guard is `v-if="visible"` over a `const visible = true`, so no checked
//!   value's type depends on the guard holding. `generate_component_prop_checks`
//!   groups the resolver call and the per-prop checks into one `if (guard)`
//!   block precisely so that a sibling prop narrowed by the guard keeps its
//!   narrowed type in both — `rows` here is `{ id: number }[] | null`, and an
//!   empty diagnostic for `:items="rows"` is what shows the grouping held;
//! * every local the resolution emits — the resolved and selected bindings, and
//!   the `__VizePropsResolver`/`__VizePropsSelector`/`__VizeResolvedProp`
//!   aliases — is read only through `typeof`. If one ever stops being read,
//!   `noUnusedLocals` turns it into a `TS6133`/`TS6196` on a clean SFC, which
//!   reaches check-server clients as an unmapped hint the same way the native
//!   element aliases did before #3443. No other test checks the prop-check path
//!   with that option on.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};
use vize_s0::String;

/// vue-tsc 3.3.4 with TypeScript 6.0.3, on this fixture and this tsconfig:
///
/// ```text
/// src/Parent.vue(9,53): error TS2322: Type 'number' is not assignable to type 'string'.
/// src/Parent.vue(10,85): error TS2322: Type 'number' is not assignable to type 'string'.
/// src/Parent.vue(11,39): error TS2322: Type 'string' is not assignable to type 'number'.
/// ```
///
/// Line 9 column 53 is the `item` of `item.id` inside the guarded usage's
/// callback. Nothing is reported for `:items="rows"` on that line, because the
/// `v-if` narrows `rows` away from `null` — so an equal list also proves the
/// resolution reads the narrowed type. The list is asserted whole under
/// `noUnusedLocals`, so an unread generated local fails here too.
#[test]
fn a_narrowing_guard_keeps_resolved_callback_props_and_leaves_no_unused_locals() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "generic-inline-callback-prop-narrowing-guard",
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
                "src/CallbackOnly.vue",
                r#"<script setup lang="ts" generic="T extends string = string">
defineProps<{ transform: (item: T) => number }>()
</script>

<template><span /></template>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
import CallbackOnly from './CallbackOnly.vue'
const groups = [[{ id: 1 }]]
const rows = Math.random() > 0.5 ? [{ id: 1 }] : null
</script>

<template>
  <Child v-if="rows" :items="rows" :pick="(item) => item.id" />
  <Child v-for="group in groups" :key="group[0].id" :items="group" :pick="(item) => item.id" />
  <CallbackOnly :transform="(item) => item.toUpperCase()" />
</template>
"#,
            ),
        ],
    );
    super::super::no_unused::write_no_unused_tsconfig(&project_root);

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };

    assert_eq!(
        snapshot,
        // The helper sorts the rendered `line:column` strings, so the
        // double-digit line numbers come before line 9.
        vec![
            (
                String::from("src/Parent.vue"),
                Some(2322),
                String::from("10:85:error Type 'number' is not assignable to type 'string'."),
            ),
            (
                String::from("src/Parent.vue"),
                Some(2322),
                String::from("11:39:error Type 'string' is not assignable to type 'number'."),
            ),
            (
                String::from("src/Parent.vue"),
                Some(2322),
                String::from("9:53:error Type 'number' is not assignable to type 'string'."),
            ),
        ]
    );
}
