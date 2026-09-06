use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};
use super::diagnostic_normalization::normalize_target_parameter_names;

#[test]
fn component_vif_guard_callback_reports_only_from_authored_guard() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "component-vif-guard-callback-single-diagnostic",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineProps<{ value: number }>()
defineEmits<{ save: [changed: boolean, invalid: boolean] }>()
defineSlots<{
  default(props: { label: string }): unknown
  caption(props: { text: string }): unknown
}>()
</script>

<template><slot label="ready" /><slot name="caption" text="done" /></template>
"#,
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
const form: any = []
const model = 1
function onSave(_changed: boolean, _invalid: boolean) {}
</script>

<template>
  <Child
    v-if="form.filter(item => item).length"
    :value="model"
    @save="(changed, invalid) => onSave(changed, invalid)"
  >
    <template #default="{ label }">{{ label.toUpperCase() }}</template>
    <template #caption="{ text }">{{ text.toUpperCase() }}</template>
  </Child>
</template>
"#,
            ),
        ],
    );

    let snapshot = normalize_target_parameter_names(snapshot_project_diagnostics(&project_root));
    let _ = std::fs::remove_dir_all(&project_root);

    assert_eq!(
        snapshot,
        Some(vec![(
            vize_s0::String::from("src/App.vue"),
            Some(7006),
            vize_s0::String::from("10:23:error Parameter 'item' implicitly has an 'any' type."),
        )]),
    );
}
