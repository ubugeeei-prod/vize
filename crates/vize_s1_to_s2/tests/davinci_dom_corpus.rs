//! Davinci P2-11 DOM corpus-runnable entry.
//!
//! This is the P1-6/P1-7 lane shape applied to the S2 DOM emitter without
//! switching the published compiler path: a committed SFC battery is compared
//! byte-for-byte against the shipped DOM template emitter, and
//! `VIZE_DAVINCI_DIFFERENTIAL_CORPUS=<dir>` widens the same comparison to every
//! SFC template block under the fixture root. The canonical fixture root fails
//! closed unless its gitlink inventory reconciles (see
//! `davinci_test_support::corpus`).

#![expect(
    clippy::disallowed_types,
    reason = "test fixtures and insta snapshots use std strings and format"
)]

mod davinci_dom_corpus_support;

use davinci_dom_corpus_support::{
    Report, assert_clean_corpus, assert_empty, compare_sfc_template, compare_sweep,
    old_lane_skip_is_allowed,
};
use vize_atelier_dom::errors::ErrorCode;

const BATTERY: &[(&str, &str)] = &[
    (
        "template_only",
        r#"<template><div class="x">{{ msg }}</div></template>"#,
    ),
    (
        "script_setup",
        r#"<script setup>const msg = "hi"</script><template><p>{{ msg }}</p></template>"#,
    ),
    (
        "slot_template",
        r#"<template><Foo><template #default="{ item }"><span>{{ item }}</span></template></Foo></template>"#,
    ),
    (
        "self_closing_non_void_html",
        r#"<template><div /><span class="x" /></template>"#,
    ),
    // A single element under `<template v-if>` is the branch block on both
    // lanes, static or not; it is never hoisted out from under the branch key.
    (
        "template_branch_single_static_element",
        r#"<template><section><template v-if="ok"><p>one</p></template><template v-else><p class="b">two</p></template></section></template>"#,
    ),
    // `v-once` on the `v-if` element caches the whole chain on both lanes.
    (
        "once_on_conditional_chain",
        r#"<template><section><input v-if="ok" v-once :value="v" /><b v-else>x</b></section></template>"#,
    ),
    (
        "once_on_conditional_component",
        r#"<template><section><Comp v-if="ok" v-once :a="a" /></section></template>"#,
    ),
    (
        "once_on_template_chain_carrier",
        r#"<template><section><template v-if="ok" v-once><p>{{ a }}</p><i>x</i></template><b v-else>y</b></section></template>"#,
    ),
    (
        "once_inside_template_branch",
        r#"<template><section><template v-if="ok"><p v-once>{{ a }}</p></template></section></template>"#,
    ),
    // The root under `<template v-if>` hoists exactly what a `v-if` element
    // does: its static descendants, and never an unused copy of its own props.
    (
        "template_branch_root_static_descendant",
        r#"<template><div><template v-if="ok"><span>{{ v }}<a href="x">more</a></span></template></div></template>"#,
    ),
    (
        "template_branch_root_static_props",
        r#"<template><div><template v-if="ok"><div flex gap-1><i class="x" /><span>{{ v }}</span></div></template></div></template>"#,
    ),
    (
        "template_branch_component_root_static_props",
        r#"<template><Foo><template v-if="ok"><i18n-t keypath="a.b">{{ v }}</i18n-t></template><i18n-t v-else keypath="a.c">{{ v }}</i18n-t></Foo></template>"#,
    ),
    (
        "conditional_slot_component_props_do_not_enable_static_cache",
        r#"<template><div><Foo><template v-if="ok" #append><Button :to="{ name: 'settings' }">{{ t('go') }}</Button></template></Foo><div id="anchor" /></div></template>"#,
    ),
    (
        "typed_local_callback_does_not_hoist_component_props",
        r#"<template><div><Form :transform="(data) => { const name: string = data.name; return name }"><input name="name" /></Form></div></template>"#,
    ),
    (
        "unstripped_ts_style_assertion_stays_dynamic",
        r#"<template><div><section :style="{ color: 'red' } as StyleValue" /><span /></div></template>"#,
    ),
    (
        "non_null_event_call_keeps_inline_handler_wrapper",
        r#"<template><div @contextmenu.prevent="options!.tippy?.show()" /></template>"#,
    ),
];

#[test]
fn dom_emit_agrees_on_sfc_templates() {
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(dom_emit_agrees_on_sfc_templates_body)
        .expect("spawn P2-11 DOM corpus thread")
        .join()
        .expect("P2-11 DOM corpus thread must not panic");
}

fn dom_emit_agrees_on_sfc_templates_body() {
    let mut report = Report::default();
    for (name, source) in BATTERY {
        compare_sfc_template(name, source, &mut report);
    }
    assert_eq!(report.templates, BATTERY.len() as u64);
    assert_eq!(report.old_error_skips, 0, "battery old-lane error skips");
    assert_empty("battery S2 refusals", &report.s2_refusals);
    assert_empty("battery divergences", &report.divergences);
    assert!(
        report.patch_fact_entries > 0,
        "battery must materialize patch facts before the corpus gate: compared={} materialized_entries={}",
        report.patch_fact_compared,
        report.patch_fact_entries,
    );

    let Some(sweep) = davinci_test_support::corpus::resolve_env_sweep() else {
        eprintln!("VIZE_DAVINCI_DIFFERENTIAL_CORPUS unset: committed battery only");
        return;
    };
    assert!(
        !sweep.files.is_empty(),
        "corpus sweep found no .vue files under {}",
        sweep.root.display()
    );

    let corpus = compare_sweep(&sweep);
    eprintln!(
        "davinci DOM corpus sweep: files={} unreadable={} parsed={} templates={} compared={} patch_fact_entries={} old_error_skips={} s2_refusals={} divergences={}",
        corpus.files,
        corpus.unreadable_count,
        corpus.parsed,
        corpus.templates,
        corpus.compared,
        corpus.patch_fact_entries,
        corpus.old_error_skips,
        corpus.s2_refusal_count,
        corpus.divergence_count,
    );
    eprintln!(
        "davinci DOM corpus refusal reasons: {:?}",
        corpus.s2_refusal_reasons
    );
    eprintln!(
        "davinci DOM corpus old-lane error reasons: {:?}",
        corpus.old_error_reasons
    );
    eprintln!(
        "davinci DOM corpus refusal samples: {:?}",
        corpus.s2_refusal_samples
    );
    assert!(
        corpus.compared > 0,
        "a corpus sweep that compares nothing proves nothing"
    );
    assert_clean_corpus(&corpus);
}

#[test]
fn nested_anchor_and_button_recoveries_are_compared_not_skipped() {
    let mut report = Report::default();
    for (name, source) in [
        (
            "nested_anchor",
            r#"<template><a href="/"><div><a href="/foo">inner</a></div></a></template>"#,
        ),
        (
            "nested_button",
            "<template><button><div><button>bbb</button></div></button></template>",
        ),
        (
            "duplicate_attribute",
            r#"<template><div class="a" class="b">duplicate</div></template>"#,
        ),
    ] {
        compare_sfc_template(name, source, &mut report);
    }

    assert_eq!(report.templates, 3);
    assert_eq!(
        report.old_error_skips, 0,
        "recoverable old-lane diagnostics should reach the DOM comparison lane: {:?}",
        report.old_error_samples
    );
    assert_eq!(
        report.s2_refusal_count, 0,
        "S2 should emit so any remaining mismatch is counted as a divergence: {:?}",
        report.s2_refusals
    );
    assert_eq!(report.compared, 3);
}

#[test]
fn unrelated_invalid_end_tag_still_blocks_old_lane_comparison() {
    let mut report = Report::default();
    compare_sfc_template(
        "stray_span_end",
        "<template><div></span></div></template>",
        &mut report,
    );

    assert_eq!(report.templates, 1);
    assert_eq!(report.compared, 0);
    assert_eq!(report.old_error_skips, 1);
    assert_eq!(report.unexpected_old_error_skips, 1);
    assert_eq!(
        report.old_error_codes,
        vec![ErrorCode::InvalidEndTag],
        "the hard invalid end tag must remain visible in skip evidence: {:?}",
        report.old_error_samples
    );
    assert_eq!(report.old_error_reasons.get("InvalidEndTag"), Some(&1));
}

#[test]
fn canonical_invalid_fixture_allowlist_is_exact() {
    assert!(old_lane_skip_is_allowed(
        "/repo/tests/_fixtures/_git/vue-manage-system/src/views/table/basetable.vue",
        &[String::from("InvalidEndTag")],
    ));
    assert!(!old_lane_skip_is_allowed(
        "/repo/tests/_fixtures/_git/vue-manage-system/src/views/table/basetable.vue",
        &[String::from("MissingEndTag")],
    ));
    assert!(!old_lane_skip_is_allowed(
        "/repo/tests/_fixtures/_git/vue-manage-system/src/views/table/other.vue",
        &[String::from("InvalidEndTag")],
    ));
}
