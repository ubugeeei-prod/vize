//! Whole-module parity controls for n8n helper order and conditional props.

#![expect(
    clippy::disallowed_types,
    reason = "whole-module differential test reports use std strings"
)]

mod davinci_dom_corpus_support;

use davinci_dom_corpus_support::{Lane, Report, compare_sfc_template_lane};

const NESTED_SLOT: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/n8n-helper-props/NestedConditionalSlot.vue.txt"
);
const CONDITIONAL_PROP: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/n8n-helper-props/ConditionalComputedProp.vue.txt"
);

fn assert_all_lanes(cases: &[(&str, &str)]) {
    for lane in [Lane::Default, Lane::Prefixed, Lane::Bindings] {
        let mut report = Report::default();
        for (name, source) in cases {
            compare_sfc_template_lane(name, source, &mut report, lane);
        }
        assert_eq!(report.files, cases.len() as u64);
        assert_eq!(report.parsed, cases.len() as u64);
        assert_eq!(report.templates, cases.len() as u64);
        assert_eq!(report.compared, cases.len() as u64);
        assert_eq!(report.old_error_skips, 0);
        assert_eq!(report.s2_refusal_count, 0, "{:?}", report.s2_refusals);
        assert_eq!(report.divergence_count, 0, "{:?}", report.divergences);
        assert_eq!(report.s2_refusals, Vec::<String>::new());
        assert_eq!(report.divergences, Vec::<String>::new());
    }
}

#[test]
fn ordinary_wrappers_preserve_authored_slot_helper_order() {
    assert_all_lanes(&[
        ("nested_conditional_slot", NESTED_SLOT),
        (
            "nested_slot_first",
            "<template><Wrapper><section><div><slot /></div><span>{{ value }}</span></section></Wrapper></template>",
        ),
        (
            "preceding_vnode_stays_first",
            "<template><Wrapper><section><span>{{ value }}</span><div><slot /></div></section></Wrapper></template>",
        ),
        (
            "preceding_sibling_stays_first",
            "<template><Wrapper><span>{{ value }}</span><div><slot /></div></Wrapper></template>",
        ),
    ]);
}

#[test]
fn conditional_computed_props_preserve_branch_object_emission() {
    assert_all_lanes(&[
        ("conditional_computed_prop", CONDITIONAL_PROP),
        (
            "ordinary_root_keeps_normalization",
            r#"<template><div :[attribute]="value" class="plain-root" /></template>"#,
        ),
        (
            "branch_with_authored_key",
            r#"<template><div v-if="visible" :key="id" :[attribute]="value" /><div v-else :[attribute]="fallback" /></template>"#,
        ),
        (
            "computed_class_or_style_branch",
            r#"<template><div v-if="visible" :[attribute]="value" :class="classes" :style="styles" /></template>"#,
        ),
        (
            "spread_branch_keeps_merge_emission",
            r#"<template><div v-if="visible" v-bind="attributes" :[attribute]="value" /></template>"#,
        ),
        (
            "loop_keeps_item_emission",
            r#"<template><div v-for="item in items" :[attribute]="item.value" /></template>"#,
        ),
    ]);
}
