use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

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
