//! Generic union props with a callback key present in only one branch.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};
use vize_s0::String;

/// A non-distributive `keyof Props<T>` omits the branch-only `pick` key and
/// silently falls back to the permissive callable type. The resolved branch's
/// callback must retain Vize's ordinary leaf-level return check.
#[test]
fn resolved_callback_prop_distributes_over_union_branches() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "generic-inline-callback-union-props",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts" generic="T extends { id: number }">
type Props<T> =
  | { kind: 'pick'; items: T[]; pick: (item: T) => string }
  | { kind: 'plain'; items: T[] }
defineProps<Props<T>>()
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
  <Child kind="pick" :items="[{ id: 1 }]" :pick="(item) => item.id" />
  <Child kind="plain" :items="[{ id: 1 }]" />
</template>
"#,
            ),
        ],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else { return };

    assert_eq!(
        snapshot,
        vec![(
            String::from("src/Parent.vue"),
            Some(2322),
            String::from("6:60:error Type 'number' is not assignable to type 'string'."),
        )]
    );
}

/// The callback key exists in both branches, so the authored discriminant must
/// select its matching signature instead of checking against their union.
#[test]
fn resolved_callback_prop_preserves_discriminated_union_correlation() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "generic-inline-callback-discriminated-union",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts" generic="T extends { id: number }">
type Props<T> =
  | { kind: 'text'; items: T[]; pick: (item: T) => string }
  | { kind: 'count'; items: T[]; pick: (item: T) => number }
defineProps<Props<T>>()
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
  <Child kind="text" :items="[{ id: 1 }]" :pick="(item) => item.id" />
  <Child kind="count" :items="[{ id: 1 }]" :pick="(item) => item.id" />
</template>
"#,
            ),
        ],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else { return };

    assert_eq!(
        snapshot,
        vec![(
            String::from("src/Parent.vue"),
            Some(2322),
            String::from("6:60:error Type 'number' is not assignable to type 'string'."),
        )]
    );
}

/// A declared `never` is a real prop type, not the missing-key sentinel. The
/// mapped callback owner must preserve it and report the authored binding.
#[test]
fn resolved_callback_prop_preserves_declared_never() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "generic-inline-callback-never-prop",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts" generic="T">
defineProps<{ value: T; pick: never }>()
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
  <Child :value="1" :pick="() => 1" />
</template>
"#,
            ),
        ],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else { return };

    assert_eq!(
        snapshot,
        vec![(
            String::from("src/Parent.vue"),
            Some(2322),
            String::from("6:22:error Type '() => number' is not assignable to type 'never'."),
        )]
    );
}

/// Selecting a branch which does not declare the authored callback must not
/// borrow the callback type from a different branch.
#[test]
fn resolved_callback_prop_rejects_a_key_missing_from_the_selected_branch() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "generic-inline-callback-missing-selected-key",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts" generic="T extends { id: number }">
type Props<T> =
  | { kind: 'pick'; items: T[]; pick: (item: T) => string }
  | { kind: 'plain'; items: T[] }
defineProps<Props<T>>()
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
  <Child kind="plain" :items="[{ id: 1 }]" :pick="() => 'x'" />
</template>
"#,
            ),
        ],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else { return };

    assert_eq!(
        snapshot,
        vec![(
            String::from("src/Parent.vue"),
            Some(2322),
            String::from("6:45:error Type '() => string' is not assignable to type 'never'."),
        )]
    );
}

/// A sibling inference failure already has its own mapped owner. Branch
/// selection must fall back to a callable context instead of adding TS7006 on
/// the callback parameter or a second callback mismatch.
#[test]
fn failed_sibling_inference_does_not_cascade_into_the_callback() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "generic-inline-callback-failed-sibling-inference",
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
  <Child :items="[{ id: 'x' }]" :pick="(item) => String(item.id)" />
</template>
"#,
            ),
        ],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else { return };

    assert_eq!(
        snapshot,
        vec![(
            String::from("src/Parent.vue"),
            Some(2322),
            String::from("6:21:error Type 'string' is not assignable to type 'number'."),
        )]
    );
}
