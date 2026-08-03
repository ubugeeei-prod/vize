//! Unknown component events must not hide authored implicit-any parameters (#3756).

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn unknown_component_event_callbacks_match_vue_tsc_implicit_any() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "unknown-component-event-callback",
        &[
            (
                "src/global-components.d.ts",
                r#"import type { DefineComponent } from "vue"
export {}

declare module "vue" {
  interface GlobalComponents {
    MkSuspense: DefineComponent<{
      p?: () => unknown
    }>
  }
}
"#,
            ),
            (
                "src/App.vue",
                r#"<template>
  <MkSuspense @resolved="(result) => void result" />
</template>
"#,
            ),
        ],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);

    assert_eq!(
        snapshot,
        // vue-tsc 3.3.4 with TypeScript 6.0.3, on the byte-identical fixture:
        // src/App.vue(2,27): error TS7006: Parameter 'result' implicitly has an 'any' type.
        Some(vec![(
            vize_carton::String::from("src/App.vue"),
            Some(7006),
            vize_carton::String::from(
                "2:27:error Parameter 'result' implicitly has an 'any' type.",
            ),
        )]),
    );
}
