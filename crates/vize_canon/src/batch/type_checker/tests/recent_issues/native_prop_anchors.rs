//! Native element prop diagnostic anchors (#3517).

use super::super::{
    create_project_case_without_node_modules, resolve_test_tsgo_binary,
    snapshot_project_diagnostics, write_test_vue_stub,
};
use crate::batch::runtime_deps::VUE_RUNTIME_DOM_STUB_TYPES;

#[test]
fn multiline_native_v_bind_anchors_at_the_attribute_name() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case_without_node_modules(
        "multiline-native-v-bind-anchor",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
const identity = (value: HTMLElement) => value
</script>

<template>
  <span
    :ref="(el: HTMLElement) => {
      identity(el)
      return undefined
    }"
  />
</template>
"#,
        )],
    );
    let node_modules = project_root.join("node_modules");
    write_test_vue_stub(&node_modules).expect("write isolated Vue stub");
    let vue_types = VUE_RUNTIME_DOM_STUB_TYPES.replace(
        "export type NativeElements = Record<string, Record<string, unknown>>;",
        "export interface NativeElements {\n  span: { ref?: (value: Element | null) => void };\n}",
    );
    std::fs::write(node_modules.join("@vue/runtime-dom/index.d.ts"), vue_types)
        .expect("pin native span ref type");

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let snapshot = snapshot.expect("type-check isolated native prop project");

    assert_eq!(
        snapshot,
        vec![(
            "src/App.vue".into(),
            Some(2322),
            "7:6:error Type '(el: HTMLElement) => undefined' is not assignable to type '(value: Element | null) => void'.\nTypes of parameters 'el' and 'value' are incompatible.\nType 'Element | null' is not assignable to type 'HTMLElement'.\nType 'null' is not assignable to type 'HTMLElement'.".into(),
        )]
    );
}
