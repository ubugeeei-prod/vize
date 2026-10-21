//! Dynamic native component unions must not leak inference-only TS2345 errors
//! into authored template diagnostics (#10026).

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn dynamic_component_union_listener_preserves_clean_diagnostics() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = super::real_vue::create_project_case(
        "dynamic-component-union-listener",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
defineProps<{ is: 'input' | 'textarea' }>()
</script>

<template>
  <component :is="is" value="text" @input="() => {}" />
</template>
"#,
        )],
    );
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "module": "ESNext",
    "moduleResolution": "bundler"
  },
  "include": ["src/**/*.vue"]
}"#,
    )
    .unwrap();

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);

    assert_eq!(
        snapshot,
        Some(Vec::new()),
        "dynamic native component unions should not report inference-only diagnostics: {snapshot:#?}"
    );
}
