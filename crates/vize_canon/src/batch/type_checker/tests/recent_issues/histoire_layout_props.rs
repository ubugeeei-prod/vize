//! Histoire's `Story` layout prop is a discriminated union. Reka UI story files
//! intentionally contain branch-exclusive fields beside a selected `type`, and
//! vue-tsc reports those excess nested keys (#5722).

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};
use vize_s0::String;

#[test]
fn story_layout_prop_reports_branch_exclusive_fields() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "histoire-layout-props",
        &[
            (
                "src/global-components.d.ts",
                r#"import type { DefineComponent } from 'vue'
export {}

declare module 'vue' {
  interface GlobalComponents {
    Story: DefineComponent<{
      layout?:
        | { type: 'grid'; width?: string | number }
        | { type: 'single'; iframe?: boolean }
    }>
  }
}
"#,
            ),
            (
                "src/Parent.vue",
                r#"<template>
  <Story :layout="{ type: 'grid', width: '100%', iframe: false }" />
  <Story :layout="{ type: 'single', width: '100%', iframe: false }" />
</template>
"#,
            ),
        ],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };

    assert_eq!(
        snapshot,
        vec![
            (
                String::from("src/Parent.vue"),
                Some(2353),
                String::from(
                    "2:50:error Object literal may only specify known properties, and 'iframe' does not exist in type '{ type: \"grid\"; width?: string | number | undefined; }'."
                ),
            ),
            (
                String::from("src/Parent.vue"),
                Some(2353),
                String::from(
                    "3:37:error Object literal may only specify known properties, and 'width' does not exist in type '{ type: \"single\"; iframe?: boolean | undefined; }'."
                ),
            ),
        ],
        "layout branch-exclusive fields must match vue-tsc diagnostics"
    );
}
