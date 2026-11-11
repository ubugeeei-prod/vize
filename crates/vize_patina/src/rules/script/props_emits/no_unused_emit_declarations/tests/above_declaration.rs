use super::lint_sfc;

#[test]
fn emit_in_a_function_declared_above_define_emits_is_used() {
    let sfc = r#"<script setup lang="ts">
const confirm = () => {
  emit("confirm");
};
const emit = defineEmits<{
  confirm: [];
  cancel: [];
}>();
const cancel = () => {
  emit("cancel");
};
</script>
<template>
  <button type="button" @click="confirm">OK</button>
  <button type="button" @click="cancel">Cancel</button>
</template>
"#;
    let result = lint_sfc(sfc);
    assert!(
        result.diagnostics.is_empty(),
        "{:?}",
        result
            .diagnostics
            .iter()
            .map(|d| d.message.as_str())
            .collect::<Vec<_>>()
    );
}
