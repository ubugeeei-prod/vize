//! Vapor SFC typecheck diagnostic anchors (#3740).

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

/// vue-tsc 3.3.4 reports the same complete diagnostic at `title`, line 3,
/// column 7. The bad value comes from a declared component prop and is used as
/// a template binding, while the explicit `Readonly` annotation avoids relying
/// on vue-tsc's incomplete inference for Vapor-mode `defineProps`.
#[test]
fn vapor_component_prop_type_error_keeps_its_authored_range() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "vapor-script-setup-type-error-anchor",
        &[(
            "src/VaporError.vue",
            r#"<script setup lang="ts" vapor>
const props: Readonly<{ count: number }> = defineProps<{ count: number }>()
const title: string = props.count
</script>

<template>
  <div :title="title" />
</template>
"#,
        )],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let snapshot = snapshot.expect("type-check Vapor SFC project");

    assert_eq!(
        snapshot,
        vec![(
            "src/VaporError.vue".into(),
            Some(2322),
            "3:7:error Type 'number' is not assignable to type 'string'.".into(),
        )]
    );
}
