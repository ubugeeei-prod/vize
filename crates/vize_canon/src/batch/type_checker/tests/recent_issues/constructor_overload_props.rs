//! Overloaded Vue constructors must check the props of the signature the
//! template arguments select (#11023). The last zero-argument overload's
//! instance `$props` is not that signature.

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};
use vize_l0::String;

const BASE: &str = r#"import { defineComponent, type SlotsType, type VNode, type PropType } from 'vue';
const VueChild = defineComponent({
  props: { value: { type: Number, required: true }, render: Function as PropType<(value: number) => string> },
  slots: Object as SlotsType<{ default(props: { item: number }): VNode[] }>,
});
type Instance = InstanceType<typeof VueChild>;
type Typed<T> = Omit<Instance, '$props' | '$slots'> & {
  readonly $props: Readonly<{ value: T }>;
  readonly $slots: { default(props: { item: T }): VNode[] };
};
"#;

const APP: &str = r#"<script setup lang="ts">
import WitnessChild from './component';
type IsAny<T> = 0 extends 1 & T ? true : false;
type IsNever<T> = [T] extends [never] ? true : false;
type IsUnknown<T> = IsAny<T> extends true ? false : unknown extends T ? keyof T extends never ? true : false : false;
type Exact<A,B> = (<T>()=>T extends A?1:2) extends (<T>()=>T extends B?1:2) ? true : false;
type SlotReport<T> = IsAny<T> extends true ? 'any' : IsNever<T> extends true ? 'never' : IsUnknown<T> extends true ? 'unknown' : Exact<T, number> extends true ? 'number' : 'other';
function slotReport<T>(_value: T): SlotReport<T> { throw new Error('witness'); }
function expectNumber(_value: 'number'): void {}
</script>
<template>
  <WitnessChild :value="1">
    <template #default="{ item }">{{ expectNumber(slotReport(item)) }}</template>
  </WitnessChild>
</template>
"#;

fn component(signature: &str) -> String {
    String::from(format!(
        "{BASE}export declare const WitnessChild: {{ {signature} }};\nexport default WitnessChild;\n"
    ))
}

fn diagnostics(
    name: &str,
    signature: &str,
    source: &str,
) -> Option<Vec<(String, Option<u32>, String)>> {
    let project = super::real_vue::create_project_case(
        name,
        &[
            ("src/component.ts", component(signature).as_str()),
            ("src/App.vue", source),
        ],
    );
    let actual = snapshot_project_diagnostics(&project);
    let _ = std::fs::remove_dir_all(project);
    actual
}

#[test]
fn last_zero_argument_overload_keeps_the_numeric_props_and_slot() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let actual = diagnostics(
        "overload-last-zero-11023",
        "new(props: { value: number }): Typed<number>; new(): Typed<string>",
        APP,
    );
    assert_eq!(actual, Some(vec![]));
}

#[test]
fn first_zero_argument_overload_keeps_the_numeric_props_and_slot() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let actual = diagnostics(
        "overload-first-zero-11023",
        "new(): Typed<string>; new(props: { value: number }): Typed<number>",
        APP,
    );
    assert_eq!(actual, Some(vec![]));
}

#[test]
fn wrong_prop_on_a_single_constructor_stays_reported() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let source = APP.replace(":value=\"1\"", ":value=\"'bad'\"");
    let actual = diagnostics("overload-single-wrong-11023", "new(): Instance", &source).unwrap();
    assert!(
        actual.iter().any(|(_, code, _)| *code == Some(2322)),
        "{actual:?}"
    );
}
