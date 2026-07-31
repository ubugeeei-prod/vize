//! Sequence expressions passed to component props (#3547).
//!
//! The generated per-prop initializer and generic props object both place a
//! prop value next to comma-delimited syntax. An authored top-level sequence
//! must stay one expression at both boundaries, and its final callback must
//! retain the child's contextual parameter type.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn non_generic_sequence_callback_prop_is_contextually_typed() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "non-generic-sequence-callback-prop",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineProps<{ transform: (value: number) => number }>()
</script>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
</script>

<template>
  <Child :transform="void 0, (value) => value" />
</template>
"#,
            ),
        ],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    assert_eq!(snapshot, Some(Vec::new()));
}

#[test]
fn generic_sequence_callback_prop_is_contextually_typed() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "generic-sequence-callback-prop",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts" generic="T">
defineProps<{ value: T; transform: (value: T) => T }>()
</script>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
</script>

<template>
  <Child :value="1" :transform="void 0, (value) => value" />
</template>
"#,
            ),
        ],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    assert_eq!(snapshot, Some(Vec::new()));
}
