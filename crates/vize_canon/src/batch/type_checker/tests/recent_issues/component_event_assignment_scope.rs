//! #4962: component event handlers assign to refs while building listener
//! bodies. Those assignments must not narrow later sibling render bindings.

use super::super::{
    create_project_case_without_node_modules, resolve_test_tsgo_binary,
    snapshot_project_diagnostics,
};

#[test]
fn component_event_assignment_does_not_narrow_render_scope() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case_without_node_modules(
        "component-event-assignment-scope",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineProps<{ modelValue?: boolean; isOpenedDialogs?: boolean }>()
defineEmits<{ "bulk-create-contents": []; "edit-contents:csv": [] }>()
</script>

<template><div /></template>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import { ref } from "vue"
import Child from "./Child.vue"

const DIALOG_STATE = { CLOSED: "closed", OPENED_A: "a", OPENED_BULK: "bulk" } as const
type DialogState = (typeof DIALOG_STATE)[keyof typeof DIALOG_STATE]
const dialogState = ref<DialogState>(DIALOG_STATE.CLOSED)
</script>

<template>
  <div>
    <Child
      :is-opened-dialogs="dialogState !== DIALOG_STATE.CLOSED"
      @edit-contents:csv="dialogState = DIALOG_STATE.OPENED_A"
      @bulk-create-contents="dialogState = DIALOG_STATE.OPENED_BULK"
    />
    <Child :model-value="dialogState === DIALOG_STATE.OPENED_A" />
  </div>
</template>
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    let relevant: Vec<_> = snapshot
        .iter()
        .filter(|(file, code, _)| file == "src/Parent.vue" && *code == Some(2367))
        .cloned()
        .collect();

    assert!(
        relevant.is_empty(),
        "component event assignments should not narrow sibling render bindings: {relevant:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
