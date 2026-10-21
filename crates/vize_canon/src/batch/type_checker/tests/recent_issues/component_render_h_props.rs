//! An imported SFC keeps its required props when passed to Vue's `h()` from a
//! TypeScript module (#9855).

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};
use super::real_vue::create_project_case;

#[test]
fn imported_sfc_h_calls_check_missing_and_wrong_props() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "component-render-h-props",
        &[
            (
                "src/Counter.vue",
                r#"<script setup lang="ts">
defineProps<{ count: number; label?: string }>();
</script>
<template><span>{{ label }}: {{ count }}</span></template>
"#,
            ),
            (
                "src/render.ts",
                r#"import { h } from "vue";
import Counter from "./Counter.vue";

export const missing = h(Counter, {});
export const wrong = h(Counter, { count: "three" });
export const valid = h(Counter, { count: 3 });
"#,
            ),
        ],
    );
    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };

    let diagnostics: Vec<_> = snapshot
        .iter()
        .map(|(file, code, message)| (file.as_str(), *code, message.as_str()))
        .collect();
    assert_eq!(
        diagnostics,
        vec![
            (
                "src/render.ts",
                Some(2769),
                "4:35:error No overload matches this call.\nThe last overload gave the following error.\nArgument of type '{}' is not assignable to parameter of type 'VNodeProps & { __v_isVNode?: undefined; [Symbol.iterator]?: undefined; } & Record<string, any> & { $props: Props; readonly __vizeRawProps?: Props | undefined; $emit: (event: string, ...args: any[]) => void; $slots: Readonly<...>; } & __VizeComponentPublicBase'.\nType '{}' is missing the following properties from type '{ $props: Props; readonly __vizeRawProps?: Props | undefined; $emit: (event: string, ...args: any[]) => void; $slots: Readonly<InternalSlots>; }': $props, $emit, $slots",
            ),
            (
                "src/render.ts",
                Some(2769),
                "5:33:error No overload matches this call.\nThe last overload gave the following error.\nArgument of type '{ count: string; }' is not assignable to parameter of type 'VNodeProps & { __v_isVNode?: undefined; [Symbol.iterator]?: undefined; } & Record<string, any> & { $props: Props; readonly __vizeRawProps?: Props | undefined; $emit: (event: string, ...args: any[]) => void; $slots: Readonly<...>; } & __VizeComponentPublicBase'.\nType '{ count: string; }' is missing the following properties from type '{ $props: Props; readonly __vizeRawProps?: Props | undefined; $emit: (event: string, ...args: any[]) => void; $slots: Readonly<InternalSlots>; }': $props, $emit, $slots",
            ),
        ],
        "only the missing and wrong h() calls may report diagnostics: {snapshot:#?}"
    );
}
