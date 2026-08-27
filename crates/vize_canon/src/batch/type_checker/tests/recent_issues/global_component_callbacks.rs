//! Contextual typing for callbacks passed to ambient global components (#3756).

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

const GLOBAL_COMPONENTS: &str = r#"import type { DefineComponent } from "vue"
export {}

declare module "vue" {
  interface GlobalComponents {
    VaInput: DefineComponent<{
      rules?: Array<(value: string) => string | boolean>
    }>
    MkSuspense: DefineComponent<{
      onResolved?: (result: { file: string }) => void
    }>
    UnresolvedSuspense: DefineComponent
  }
}
"#;

#[test]
fn ambient_global_component_array_callbacks_are_contextually_typed() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "ambient-global-component-array-callbacks",
        &[
            ("src/global-components.d.ts", GLOBAL_COMPONENTS),
            (
                "src/App.vue",
                r#"<template>
  <VaInput :rules="[(value) => value.length > 0 || 'required']" />
  <VaInput :rules="[(value) => value.missing]" />
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
        // src/App.vue(3,38): error TS2339: Property 'missing' does not exist on type 'string'.
        Some(vec![(
            vize_s0::String::from("src/App.vue"),
            Some(2339),
            vize_s0::String::from("3:38:error Property 'missing' does not exist on type 'string'.",),
        )]),
    );
}

#[test]
fn ambient_global_component_event_callbacks_are_contextually_typed() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "ambient-global-component-event-callbacks",
        &[
            ("src/global-components.d.ts", GLOBAL_COMPONENTS),
            (
                "src/App.vue",
                r#"<template>
  <MkSuspense @resolved="(result) => result.file.length" />
  <MkSuspense @resolved="(result) => result.missing" />
</template>
"#,
            ),
        ],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    assert_eq!(
        snapshot,
        Some(vec![(
            vize_s0::String::from("src/App.vue"),
            Some(2339),
            vize_s0::String::from(
                "3:45:error Property 'missing' does not exist on type '{ file: string; }'.",
            ),
        )]),
    );
}

#[test]
fn unknown_global_component_array_callbacks_remain_implicit_any() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "unknown-global-component-array-callback",
        &[(
            "src/App.vue",
            r#"<template>
  <UnknownInput :rules="[(value) => value]" />
</template>
"#,
        )],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);

    assert_eq!(
        snapshot,
        // vue-tsc 3.3.4 with TypeScript 6.0.3, on the byte-identical fixture:
        // src/App.vue(2,27): error TS7006: Parameter 'value' implicitly has an 'any' type.
        Some(vec![(
            vize_s0::String::from("src/App.vue"),
            Some(7006),
            vize_s0::String::from("2:27:error Parameter 'value' implicitly has an 'any' type.",),
        )]),
    );
}

#[test]
fn ambient_global_component_unresolved_event_callbacks_remain_implicit_any() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "ambient-global-component-unresolved-event-callback",
        &[
            ("src/global-components.d.ts", GLOBAL_COMPONENTS),
            (
                "src/App.vue",
                r#"<template>
  <UnresolvedSuspense v-slot="{ result }" @resolved="(result) => result.file" />
</template>
"#,
            ),
        ],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);

    assert_eq!(
        snapshot,
        Some(vec![(
            vize_s0::String::from("src/App.vue"),
            Some(7006),
            vize_s0::String::from("2:55:error Parameter 'result' implicitly has an 'any' type."),
        )]),
    );
}
