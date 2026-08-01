//! Optional boolean props are normalized when passed through templates (#2719).

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn issue_2719_normalizes_optional_boolean_props_in_template() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "issue-2719-optional-boolean-template-prop",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineProps<{ present: boolean }>();
</script>

<template>
  <span />
</template>
"#,
            ),
            (
                "src/Foo.vue",
                r#"<script setup lang="ts">
import Child from "./Child.vue";

defineProps<{ loading?: boolean }>();
</script>

<template>
  <Child :present="loading" />
</template>
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.iter().all(|(file, code, message)| {
            !(file == "src/Foo.vue"
                && *code == Some(2322)
                && message.contains("boolean | undefined"))
        }),
        "optional boolean props should be normalized in templates, got: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
