//! Davinci P2-11 DOM corpus-runnable entry under `prefix_identifiers`
//! (installment 85): the same committed battery and
//! `VIZE_DAVINCI_DIFFERENTIAL_CORPUS=<dir>` widening as
//! `davinci_dom_corpus`, with both lanes compiled under
//! `prefix_identifiers: true`.

#![allow(clippy::disallowed_macros, clippy::disallowed_types)]

mod davinci_dom_corpus_support;

use davinci_dom_corpus_support::{
    Lane, Report, assert_clean_corpus, assert_empty, compare_sfc_template_lane, compare_sweep_lane,
};

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
        r#"<template><Foo><template #default="{ item }"><span>{{ item }}{{ other }}</span></template></Foo></template>"#,
    ),
    (
        "handlers_and_loops",
        r#"<template><ul><li v-for="item in items" :key="item.id" @click="select(item, extra)">{{ item.label }}</li></ul></template>"#,
    ),
];

#[test]
fn prefixed_dom_emit_agrees_on_sfc_templates() {
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(prefixed_dom_emit_agrees_on_sfc_templates_body)
        .expect("spawn P2-11 prefixed DOM corpus thread")
        .join()
        .expect("P2-11 prefixed DOM corpus thread must not panic");
}

fn prefixed_dom_emit_agrees_on_sfc_templates_body() {
    let mut report = Report::default();
    for (name, source) in BATTERY {
        compare_sfc_template_lane(name, source, &mut report, Lane::Prefixed);
    }
    assert_eq!(report.templates, BATTERY.len() as u64);
    assert_eq!(report.old_error_skips, 0, "battery old-lane error skips");
    assert_empty("battery S2 refusals", &report.s2_refusals);
    assert_empty("battery divergences", &report.divergences);

    let Some(sweep) = davinci_test_support::corpus::resolve_env_sweep() else {
        eprintln!("VIZE_DAVINCI_DIFFERENTIAL_CORPUS unset: committed battery only");
        return;
    };
    assert!(
        !sweep.files.is_empty(),
        "corpus sweep found no .vue files under {}",
        sweep.root.display()
    );

    let corpus = compare_sweep_lane(&sweep, Lane::Prefixed);
    eprintln!(
        "davinci prefixed DOM corpus sweep: scope={} files={} unreadable={} parsed={} templates={} compared={} old_error_skips={} s2_refusals={} divergences={}",
        sweep.scope_label(),
        corpus.files,
        corpus.unreadable_count,
        corpus.parsed,
        corpus.templates,
        corpus.compared,
        corpus.old_error_skips,
        corpus.s2_refusal_count,
        corpus.divergence_count,
    );
    eprintln!(
        "davinci prefixed DOM corpus refusal reasons: {:?}",
        corpus.s2_refusal_reasons
    );
    eprintln!(
        "davinci prefixed DOM corpus old-lane error reasons: {:?}",
        corpus.old_error_reasons
    );
    eprintln!(
        "davinci prefixed DOM corpus refusal samples: {:?}",
        corpus.s2_refusal_samples
    );
    assert!(
        corpus.compared > 0,
        "a corpus sweep that compares nothing proves nothing"
    );
    assert_clean_corpus(&corpus);
}
