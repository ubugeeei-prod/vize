//! An authored `Pick` is legal TypeScript. Inline virtual-TS helpers must not
//! resolve their lib `Pick<T, K>` instantiations to that alias (#11851).

use vize_canon::{SfcTypeCheckOptions, type_check_sfc};
use vize_l0::{String, cstr};

const AUTHORED_PICK: &str = r#"<script setup lang="ts">
import { ref } from "vue"
declare const loose: any
type Function = string
type Pick<T> = T
const globalThis = { Function: "label" as Function }
const named = (value: Pick<string>) => value
const handler = ref(named)
</script>
<template>
<component :is="loose" @pick="handler" />
<span>{{ globalThis.Function }}</span>
</template>
"#;

const SUFFIXED_PICK: &str = r#"<script setup lang="ts">
type Pick<T> = T
type __vizePick<T> = T
declare const loose: any
const handler = (value: string) => value
</script>
<template>
<component :is="loose" @pick="handler" />
</template>
"#;

const UNCAPTURED: &str = r#"<script setup lang="ts">
import { ref } from "vue"
declare const loose: any
const named = (value: string) => value
const handler = ref(named)
</script>
<template>
<component :is="loose" @pick="handler" />
</template>
"#;

const CAPTURED_HELPERS: &[&str] = &[
    "type $RequiredKeys<T> = { [K in keyof T]-?: {} extends Pick<T, K> ? never : K }[keyof T];",
    "type $ReplaceComponentProps<T, P> = Omit<T, '$props'> & Pick<T, Exclude<$LiteralKeys<T>, '$props'>> & { $props: P };",
    "type $ComponentPublicBase<I = import('vue').ComponentPublicInstance> = Omit<I, '$props' | '$emit' | '$slots'> & Pick<I, Exclude<$LiteralKeys<I>, '$props' | '$emit' | '$slots'>>;",
    "type $EmitRequiredKeys<T> = { [K in keyof T]-?: {} extends Pick<T, K> ? never : K }[keyof T];",
    "type $EmitRequiredFields<T> = T extends unknown ? {} extends T ? Pick<T, never> : Pick<T, $EmitRequiredKeys<T>> : never;",
    "type $OverloadProps<TOverload> = Pick<TOverload, keyof TOverload>;",
];

fn virtual_ts(source: &str) -> String {
    type_check_sfc(
        source,
        &SfcTypeCheckOptions::new("RefAndNames.vue").with_virtual_ts(),
    )
    .virtual_ts
    .expect("virtual TypeScript")
}

fn owned_helpers(alias: &str) -> [String; 6] {
    [
        cstr!(
            "type $RequiredKeys<T> = {{ [K in keyof T]-?: {{}} extends {alias}<T, K> ? never : K }}[keyof T];"
        ),
        cstr!(
            "type $ReplaceComponentProps<T, P> = Omit<T, '$props'> & {alias}<T, Exclude<$LiteralKeys<T>, '$props'>> & {{ $props: P }};"
        ),
        cstr!(
            "type $ComponentPublicBase<I = import('vue').ComponentPublicInstance> = Omit<I, '$props' | '$emit' | '$slots'> & {alias}<I, Exclude<$LiteralKeys<I>, '$props' | '$emit' | '$slots'>>;"
        ),
        cstr!(
            "type $EmitRequiredKeys<T> = {{ [K in keyof T]-?: {{}} extends {alias}<T, K> ? never : K }}[keyof T];"
        ),
        cstr!(
            "type $EmitRequiredFields<T> = T extends unknown ? {{}} extends T ? {alias}<T, never> : {alias}<T, $EmitRequiredKeys<T>> : never;"
        ),
        cstr!("type $OverloadProps<TOverload> = {alias}<TOverload, keyof TOverload>;"),
    ]
}

fn assert_owned_pick(virtual_ts: &str, alias: &str) {
    let declaration = cstr!("type {alias}<T, K extends keyof T> = {{ [P in K]: T[P] }};");
    assert_eq!(
        virtual_ts.matches(declaration.as_str()).count(),
        1,
        "one homomorphic lib Pick, not one copy per helper chunk:\n{declaration}"
    );
    for helper in owned_helpers(alias) {
        assert_eq!(
            virtual_ts.matches(helper.as_str()).count(),
            1,
            "missing owned helper {helper}"
        );
    }
    for helper in CAPTURED_HELPERS {
        assert!(
            !virtual_ts.contains(helper),
            "inline helper still instantiates unqualified Pick: {helper}"
        );
    }
}

#[test]
fn authored_pick_alias_does_not_capture_inline_helper_references() {
    let virtual_ts = virtual_ts(AUTHORED_PICK);
    for authored in [
        "type Function = string",
        "type Pick<T> = T",
        "const globalThis = { Function: \"label\" as Function }",
        "const named = (value: Pick<string>) => value",
        "const handler = ref(named)",
        "globalThis.Function",
    ] {
        assert!(
            virtual_ts.contains(authored),
            "authored fixture text changed: {authored}"
        );
    }
    assert_owned_pick(&virtual_ts, "__vizePick");
}

#[test]
fn authored_pick_alias_keeps_a_fresh_name_when_the_owned_alias_is_taken() {
    let virtual_ts = virtual_ts(SUFFIXED_PICK);
    assert!(virtual_ts.contains("type Pick<T> = T"));
    assert!(virtual_ts.contains("type __vizePick<T> = T"));
    assert!(!virtual_ts.contains("type __vizePick<T, K extends keyof T>"));
    assert_owned_pick(&virtual_ts, "__vizePick_1");
}

#[test]
fn components_without_an_authored_pick_keep_the_builtin_spelling() {
    let virtual_ts = virtual_ts(UNCAPTURED);
    assert!(!virtual_ts.contains("__vizePick"));
    for helper in CAPTURED_HELPERS {
        assert_eq!(
            virtual_ts.matches(helper).count(),
            1,
            "builtin Pick spelling changed: {helper}"
        );
    }
}
