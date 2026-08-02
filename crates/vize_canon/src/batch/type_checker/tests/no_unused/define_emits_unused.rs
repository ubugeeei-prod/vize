use super::super::{
    create_project_case_without_node_modules, resolve_test_tsgo_binary,
    snapshot_project_diagnostics,
};
use super::write_no_unused_tsconfig;

#[test]
fn define_emits_no_unused_matrix_preserves_only_real_template_usage() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case_without_node_modules(
        "define-emits-template-dollar-emit-no-unused",
        &[
            (
                "src/DollarEmit.vue",
                r#"<script setup lang="ts">
const emit = defineEmits<{ click: [] }>()
const unusedLocal = 1
</script>

<template><button @click="$emit('click')" /></template>
"#,
            ),
            (
                "src/RenamedEmit.vue",
                r#"<script setup lang="ts">
const dispatch = defineEmits<{ save: [] }>()
</script>

<template><button @click="$emit('save')" /></template>
"#,
            ),
            (
                "src/DirectTemplateEmit.vue",
                r#"<script setup lang="ts">
const emit = defineEmits<{ click: [] }>()
</script>

<template><button @click="emit('click')" /></template>
"#,
            ),
            (
                "src/DirectScriptEmit.vue",
                r#"<script setup lang="ts">
const emit = defineEmits<{ ready: [] }>()
emit('ready')
</script>

<template><main /></template>
"#,
            ),
            (
                "src/UnusedEmit.vue",
                r#"<script setup lang="ts">
const emit = defineEmits<{ click: [] }>()
</script>

<template><button /></template>
"#,
            ),
            (
                "src/OptionsApi.vue",
                r#"<script lang="ts">
const emit = 1

export default {
  emits: ['click'],
}
</script>

<template><button @click="$emit('click')" /></template>
"#,
            ),
        ],
    );
    write_no_unused_tsconfig(&project_root);

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    for (file, binding) in [
        ("src/DollarEmit.vue", "emit"),
        ("src/RenamedEmit.vue", "dispatch"),
        ("src/DirectTemplateEmit.vue", "emit"),
        ("src/DirectScriptEmit.vue", "emit"),
    ] {
        assert!(
            !has_unused_binding(&snapshot, file, binding),
            "{file} should consume `{binding}`, got: {snapshot:#?}"
        );
    }
    for (file, binding) in [
        ("src/DollarEmit.vue", "unusedLocal"),
        ("src/UnusedEmit.vue", "emit"),
        ("src/OptionsApi.vue", "emit"),
    ] {
        assert!(
            has_unused_binding(&snapshot, file, binding),
            "{file} should still report truly unused `{binding}`, got: {snapshot:#?}"
        );
    }

    let _ = std::fs::remove_dir_all(&project_root);
}

fn has_unused_binding(
    snapshot: &[(vize_carton::String, Option<u32>, vize_carton::String)],
    file: &str,
    binding: &str,
) -> bool {
    snapshot.iter().any(|(candidate_file, code, message)| {
        candidate_file == file && *code == Some(6133) && message.contains(binding)
    })
}
