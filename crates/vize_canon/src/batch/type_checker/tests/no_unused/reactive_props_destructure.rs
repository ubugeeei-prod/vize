use super::super::{
    create_project_case_without_node_modules, resolve_test_tsgo_binary,
    snapshot_project_diagnostics,
};
use super::write_no_unused_tsconfig;

#[test]
fn unused_reactive_props_destructure_matches_vue_tsc() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case_without_node_modules(
        "reactive-props-destructure-no-unused-locals",
        &[(
            "src/MyBadge.vue",
            r#"<script setup lang="ts">
const { label, color = "gray", ...rest } = defineProps<{ label: string; color?: string; title: string }>()
const unusedLocal = 1
</script>

<template>
  <span>{{ label }} {{ rest.title }}</span>
</template>
"#,
        )],
    );
    write_no_unused_tsconfig(&project_root);

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    let unused: Vec<_> = snapshot
        .iter()
        .filter(|(file, code, _)| file == "src/MyBadge.vue" && *code == Some(6133))
        .collect();
    assert_eq!(
        unused
            .iter()
            .map(|(_, _, message)| message.as_str())
            .collect::<Vec<_>>(),
        [
            "2:16:error 'color' is declared but its value is never read.",
            "3:7:error 'unusedLocal' is declared but its value is never read.",
        ],
        "expected the complete vue-tsc diagnostic set: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
