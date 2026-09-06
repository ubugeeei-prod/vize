//! Misskey's `MkForm.vue` indexes a required `defineModel<Record<string, any>>`
//! from a template `v-for` key. The model helper is Vize-owned, so the template
//! ref unwrap must recognize that helper before it falls back to Vue's `Ref`.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn define_model_record_indexes_with_template_v_for_keys() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "define-model-template-record-index",
        &[
            (
                "src/Field.vue",
                r#"<script setup lang="ts">
defineProps<{ modelValue: unknown }>()
defineEmits<{ "update:modelValue": [value: unknown] }>()
</script>
<template><input /></template>
"#,
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import Field from './Field.vue'

type FormEntry = {
  hidden?: (values: Record<string, any>) => boolean
  type: 'text' | 'file'
}

defineProps<{ form: Record<string, FormEntry> }>()

const values = defineModel<Record<string, any>>({ required: true })
</script>

<template>
  <template v-for="v, k in form" :key="k">
    <Field v-if="!v.hidden?.(values) && v.type === 'text'" v-model="values[k]" />
    <button v-else @click="values[k] = 'file'">set</button>
  </template>
</template>
"#,
            ),
        ],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);

    assert_eq!(snapshot, Some(Vec::new()));
}
