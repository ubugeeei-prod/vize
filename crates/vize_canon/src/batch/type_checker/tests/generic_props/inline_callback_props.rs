//! Inline callback props on a generic child (#3446).
//!
//! A generic child's static per-prop alias resolves to `unknown`; the generic
//! type is instantiated only by its whole-props call. Inline arrows therefore
//! need the child's resolver result for both contextual typing and the same
//! leaf diagnostic anchor as vue-tsc. The callable fallback remains for
//! components that do not expose that resolver.

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
                String::from("6:48:error Type 'number' is not assignable to type 'string'."),
            ),
            (
                String::from("src/Parent.vue"),
                Some(2322),
                String::from("7:21:error Type 'string' is not assignable to type 'number'."),
            ),
        ]
    );
}

/// A parameter type may contain an arrow before the callback's outer arrow.
/// The prop classifier must find the outer arrow so the standalone per-prop
/// check contextually types the following `value` parameter instead of
/// reporting `TS7006` on code the generic child's checker types.
#[test]
fn inline_callback_prop_with_nested_type_arrow_is_contextually_typed() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "generic-inline-callback-prop-with-nested-type-arrow",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts" generic="T">
defineProps<{ value: T; apply: (fn: (value: T) => T, value: T) => T }>()
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
  <Child :value="1" :apply="(fn: (x: number) => number, value) => fn(value)" />
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

    assert_eq!(snapshot, Vec::new());
}

/// Delimiters inside a regex default value are not parameter-list structure.
/// The direct callback must still receive generic-prop contextual typing for
/// its following untyped parameter.
#[test]
fn inline_callback_prop_with_regex_default_is_contextually_typed() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "generic-inline-callback-prop-with-regex-default",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts" generic="T">
defineProps<{
  value: T
  apply: (pattern: RegExp, value: T) => T
  calculate: (pattern: number, value: T) => T
  inspect: (fn: () => void, value: T) => T
  choose: (value: T) => T
  sequence: (value: T) => T
}>()
</script>

<template><span /></template>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
let foo = 1
const bar = 2
const ok = true
const target = ')'
</script>

<template>
  <Child
    :value="1"
    :apply="(pattern = /* default */ /\)/, value) /* callback */ => value"
    :calculate="(pattern = foo++ / bar, value) => value"
    :inspect="(fn = () => { if (ok) /\)/.test(target) }, value) => value"
    :choose="ok ? (value) => value : (value) => value"
    :sequence="(void 0, (value) => value)"
  />
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

    assert_eq!(snapshot, Vec::new());
}

/// A nested arrow does not make the prop value itself callable. This is the
/// shape used by Misskey's `<MkTl :events="jobs.map((job) => …)" />`: treating
/// the inner `map` callback as the prop value made the array fail against the
/// callable fallback added for genuine inline callback props.
#[test]
fn array_expression_with_nested_callback_is_not_classified_as_a_callback_prop() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "generic-array-prop-with-nested-callback",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts" generic="T extends { id: number }">
defineProps<{ events: T[] }>()
</script>

<template><span /></template>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
const jobs = [{ id: 1, timestamp: 2 }]
</script>

<template>
  <Child :events="jobs.map((job) => ({ id: job.id }))" />
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
        Vec::new(),
        "nested callbacks must keep the array prop type"
    );
}

#[test]
fn non_generic_child_callback_prop_and_event_inference_stay_contextual() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "non-generic-inline-callback-prop-event",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineProps<{
  fileId: string
  validate?: (file: { id: string }) => boolean | Promise<boolean>
}>()
defineEmits<{
  update: [file: { id: string }]
}>()
</script>

<template><span /></template>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
let selected = ''
</script>

<template>
  <Child
    file-id="avatar"
    :validate="async f => f.id.length > 0"
    @update="f => selected = f.id"
  />
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

    assert_eq!(snapshot, Vec::new());
}

/// vue-tsc 3.3.4 with TypeScript 6.0.3 reports the three callback-body errors
/// below verbatim. This covers the two scopes where the resolved prop type must
/// remain local and a generic inferred only through its callback constraint.
#[test]
fn resolved_callback_props_match_vue_tsc_in_guarded_and_loop_scopes() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "generic-inline-callback-prop-scopes",
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
const visible = true
const groups = [[{ id: 1 }]]
</script>

<template>
  <Child v-if="visible" :items="[{ id: 1 }]" :pick="(item) => item.id" />
  <Child v-for="group in groups" :key="group[0].id" :items="group" :pick="(item) => item.id" />
  <CallbackOnly :transform="(item) => item.toUpperCase()" />
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
        // The snapshot helper sorts the rendered `line:column` strings, so
        // double-digit line numbers precede line 9 here.
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
                String::from("9:63:error Type 'number' is not assignable to type 'string'."),
            ),
        ]
    );
}
