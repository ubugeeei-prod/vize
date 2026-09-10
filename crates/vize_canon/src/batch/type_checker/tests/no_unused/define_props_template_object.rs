use super::super::{
    create_project_case_without_node_modules, resolve_test_tsgo_binary,
    snapshot_project_diagnostics,
};
use super::write_no_unused_tsconfig;

#[test]
fn define_props_result_binding_is_used_when_template_reads_props_object() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case_without_node_modules(
        "define-props-result-template-props-object-no-unused-locals",
        &[
            (
                "src/App.vue",
                r#"<script setup lang="ts">
interface Props {
  cols?: number
}

const props = defineProps<Props>()
const unusedLocal = 1
</script>

<template>
  <div>{{ props.cols }}</div>
</template>
"#,
            ),
            (
                "src/WithDefaults.vue",
                r#"<script setup lang="ts">
interface Props {
  cols?: number
}

const props = withDefaults(defineProps<Props>(), {
  cols: 1,
})
const unusedLocal = 1
</script>

<template>
  <div :class="[props.cols]">{{ props.cols }}</div>
</template>
"#,
            ),
        ],
    );
    write_no_unused_tsconfig(&project_root);

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    for file in ["src/App.vue", "src/WithDefaults.vue"] {
        assert!(
            !has_unused_binding(&snapshot, file, "props"),
            "{file} should count template `props.*` reads for defineProps, got: {snapshot:#?}"
        );
        assert!(
            has_unused_binding(&snapshot, file, "unusedLocal"),
            "{file} should still report unrelated TS6133, got: {snapshot:#?}"
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
