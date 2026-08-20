//! Open slot index signatures should not make every possible name required.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn open_slot_index_signature_does_not_require_every_possible_slot_name() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "open-slot-index-signature",
        &[
            (
                "src/OpenSlots.vue",
                r#"<script setup lang="ts">
defineSlots<{
  header(props: { title: string }): unknown;
  [name: string]: (props: { title: string }) => unknown;
}>();
</script>

<template>
  <slot name="header" title="ready" />
</template>
"#,
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import OpenSlots from './OpenSlots.vue';

function takesString(value: string) {
  return value;
}
</script>

<template>
  <OpenSlots>
    <template #header="{ title }">{{ takesString(title) }}</template>
  </OpenSlots>
</template>
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };
    let _ = std::fs::remove_dir_all(&project_root);

    assert!(
        snapshot.is_empty(),
        "open slot index signatures should not require every possible slot name: {snapshot:#?}"
    );
}
