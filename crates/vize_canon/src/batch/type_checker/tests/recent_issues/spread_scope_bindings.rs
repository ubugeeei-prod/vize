//! Reserved-name template locals in component prop spreads.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

/// The parent also owns a numeric `as` prop. Rewriting the loop-local `as` to
/// that outer prop would make the usage silently pass; preserving the local
/// runtime string must expose the child's number mismatch.
#[test]
fn v_for_reserved_local_keeps_its_runtime_type_in_a_spread() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "vbind-spread-v-for-reserved-local",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineProps<{ as: number }>()
</script>

<template><span /></template>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
defineProps<{ as: number }>()
const items = ['runtime-string']
</script>

<template>
  <Child v-for="as in items" v-bind="{ as }" />
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
        snapshot.len(),
        1,
        "expected one child prop mismatch: {snapshot:?}"
    );
    assert_eq!(snapshot[0].0, "src/Parent.vue");
    assert_eq!(snapshot[0].1, Some(2345));
    assert!(
        snapshot[0]
            .2
            .contains("Type 'string' is not assignable to type 'number'"),
        "the loop-local runtime type must reach the child check: {snapshot:?}"
    );
}
