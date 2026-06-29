use super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn batch_type_checker_exposes_generic_props_inherited_from_pick() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "generic-setup-props-inherited-pick-template",
        &[(
            "src/Foo.vue",
            r#"<script lang="ts">
interface BaseProps {
  required?: boolean;
}

export interface FooProps<T = string> extends Pick<BaseProps, "required"> {
  value?: T;
}
</script>

<script setup lang="ts" generic="T = string">
defineProps<FooProps<T>>();
</script>

<template>
  <input :required />
</template>
"#,
        )],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.iter().all(|(file, code, message)| {
            !(file == "src/Foo.vue" && *code == Some(2304) && message.contains("required"))
        }),
        "inherited type-based props should be exposed to generic SFC templates, got: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
