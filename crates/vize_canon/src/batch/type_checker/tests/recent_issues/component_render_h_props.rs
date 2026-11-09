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
                "4:35:error No overload matches this call.\nThe last overload gave the following error.\nArgument of type '{}' is not assignable to parameter of type 'VNodeProps & { __v_isVNode?: undefined; [Symbol.iterator]?: undefined; } & Record<string, any> & { $props: Props; readonly __vizeRawProps?: Props | undefined; $emit: (event: string, ...args: any[]) => void; $slots: Readonly<...>; } & ComponentCustomProperties & Pick<...>'.\nType '{}' is missing the following properties from type '{ $props: Props; readonly __vizeRawProps?: Props | undefined; $emit: (event: string, ...args: any[]) => void; $slots: Readonly<InternalSlots>; }': $props, $emit, $slots",
            ),
            (
                "src/render.ts",
                Some(2769),
                "5:33:error No overload matches this call.\nThe last overload gave the following error.\nArgument of type '{ count: string; }' is not assignable to parameter of type 'VNodeProps & { __v_isVNode?: undefined; [Symbol.iterator]?: undefined; } & Record<string, any> & { $props: Props; readonly __vizeRawProps?: Props | undefined; $emit: (event: string, ...args: any[]) => void; $slots: Readonly<...>; } & ComponentCustomProperties & Pick<...>'.\nType '{ count: string; }' is missing the following properties from type '{ $props: Props; readonly __vizeRawProps?: Props | undefined; $emit: (event: string, ...args: any[]) => void; $slots: Readonly<InternalSlots>; }': $props, $emit, $slots",
            ),
        ],
        "only the missing and wrong h() calls may report diagnostics: {snapshot:#?}"
    );
}

#[test]
fn capture_free_instance_keeps_authored_names_and_public_member_types() {
    if resolve_test_tsgo_binary().is_none() {
        assert!(
            std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none(),
            "required capture-free instance control cannot skip the native runtime"
        );
        return;
    }
    macro_rules! fixture {
        ($name:literal) => {
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../tests/_fixtures/differential/typechecker/",
                "capture-free-component-instance-11699/",
                $name,
                ".txt"
            ))
        };
    }
    let project_root = create_project_case(
        "capture-free-instance",
        &[
            ("src/Capture.vue", fixture!("Capture.vue")),
            ("src/authored.ts", fixture!("authored.ts")),
            ("src/globals.d.ts", fixture!("globals.d.ts")),
            ("src/render.ts", fixture!("render.ts")),
            ("tsconfig.json", fixture!("tsconfig.json")),
        ],
    );
    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let snapshot = snapshot.expect("native capture-free instance control must return diagnostics");
    let diagnostics: Vec<_> = snapshot
        .iter()
        .map(|(file, code, message)| (file.as_str(), *code, message.as_str()))
        .collect();
    assert_eq!(
        diagnostics,
        vec![
            (
                "src/render.ts",
                Some(2322),
                "7:14:error Type 'number' is not assignable to type 'boolean'.",
            ),
            (
                "src/render.ts",
                Some(2345),
                "8:50:error Argument of type 'string' is not assignable to parameter of type 'number'.",
            ),
        ],
        "valid authored names, exposed members, globals and h() calls must remain diagnostic-free: {snapshot:#?}"
    );
}
