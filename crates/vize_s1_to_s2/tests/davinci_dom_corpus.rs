//! Davinci P2-11 DOM corpus-runnable entry.
//!
//! This is the P1-6/P1-7 lane shape applied to the S2 DOM emitter without
//! switching the published compiler path: a committed SFC battery is compared
//! byte-for-byte against the shipped DOM template emitter, and
//! `VIZE_DAVINCI_DIFFERENTIAL_CORPUS=<dir>` widens the same comparison to every
//! SFC template block under the fixture root. The canonical fixture root fails
//! closed unless its gitlink inventory reconciles (see
//! `davinci_test_support::corpus`).

#![allow(clippy::disallowed_macros, clippy::disallowed_types)]

use std::{collections::BTreeMap, fs};

use vize_atelier_dom::errors::ErrorCode;
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_s0::Allocator;
use vize_s1_to_s2::{EmitError, emit_dom_source};

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
];

#[derive(Default)]
struct Report {
    files: u64,
    unreadable_count: u64,
    parsed: u64,
    templates: u64,
    compared: u64,
    old_error_skips: u64,
    s2_refusal_count: u64,
    divergence_count: u64,
    old_error_codes: Vec<ErrorCode>,
    unreadable: Vec<String>,
    old_error_samples: Vec<String>,
    s2_refusal_reasons: BTreeMap<&'static str, u64>,
    s2_refusal_samples: BTreeMap<&'static str, Vec<String>>,
    s2_refusals: Vec<String>,
    divergences: Vec<String>,
}

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

    let Some(sweep) = davinci_test_support::corpus::resolve_env_sweep() else {
        eprintln!("VIZE_DAVINCI_DIFFERENTIAL_CORPUS unset: committed battery only");
        return;
    };
    assert!(
        !sweep.files.is_empty(),
        "corpus sweep found no .vue files under {}",
        sweep.root.display()
    );

    let mut corpus = Report::default();
    for file in &sweep.files {
        let source = match fs::read_to_string(file) {
            Ok(source) => source,
            Err(error) => {
                corpus.unreadable_count += 1;
                if corpus.unreadable.len() < 20 {
                    corpus
                        .unreadable
                        .push(format!("{}: {error}", file.display()));
                }
                continue;
            }
        };
        let context = file.to_string_lossy();
        compare_sfc_template(context.as_ref(), &source, &mut corpus);
    }
    eprintln!(
        "davinci DOM corpus sweep: files={} unreadable={} parsed={} templates={} compared={} old_error_skips={} s2_refusals={} divergences={}",
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
        "davinci DOM corpus refusal reasons: {:?}",
        corpus.s2_refusal_reasons
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

fn compare_sfc_template(name: &str, source: &str, report: &mut Report) {
    report.files += 1;
    let Ok(descriptor) = parse_sfc(source, SfcParseOptions::default()) else {
        return;
    };
    report.parsed += 1;
    let Some(template) = descriptor.template.as_ref() else {
        return;
    };
    report.templates += 1;

    let old_allocator = Allocator::new();
    let (_, errors, old) = vize_atelier_dom::compile_template(&old_allocator, &template.content);
    let blocking_errors: Vec<_> = errors
        .iter()
        .filter(|error| !error.is_compatibility_notice())
        .collect();
    if !blocking_errors.is_empty() {
        report.old_error_skips += 1;
        if report.old_error_codes.len() < 20 {
            report
                .old_error_codes
                .extend(blocking_errors.iter().map(|error| error.code));
        }
        if report.old_error_samples.len() < 20 {
            report.old_error_samples.push(format!(
                "{name}: {} old-lane blocking errors: {blocking_errors:?}",
                blocking_errors.len()
            ));
        }
        return;
    }
    let old = format!("{}\n{}", old.preamble, old.code);

    let new_allocator = Allocator::new();
    let new = match emit_dom_source(&new_allocator, &template.content) {
        Ok(emit) => emit.assembled(),
        Err(error) => {
            report.s2_refusal_count += 1;
            let reason = refusal_reason(&error);
            *report.s2_refusal_reasons.entry(reason).or_default() += 1;
            let samples = report.s2_refusal_samples.entry(reason).or_default();
            if samples.len() < 5 {
                samples.push(format!("{name}: {error:?}"));
            }
            if report.s2_refusals.len() < 20 {
                report.s2_refusals.push(format!("{name}: {error:?}"));
            }
            return;
        }
    };

    report.compared += 1;
    if old != new {
        report.divergence_count += 1;
        if report.divergences.len() < 20 {
            report.divergences.push(format!(
                "{name}: old_len={} new_len={} first_diff={} old_window={} new_window={}",
                old.len(),
                new.len(),
                first_diff(&old, &new),
                mismatch_window(&old, &new),
                mismatch_window(&new, &old)
            ));
        }
    }
}

fn refusal_reason(error: &EmitError) -> &'static str {
    error.reason().map_or("diagnostics", |reason| reason.code())
}

fn preview(source: &str) -> String {
    source
        .lines()
        .take(4)
        .collect::<Vec<_>>()
        .join("\\n")
        .chars()
        .take(320)
        .collect()
}

fn first_diff(left: &str, right: &str) -> usize {
    left.as_bytes()
        .iter()
        .zip(right.as_bytes())
        .position(|(left, right)| left != right)
        .unwrap_or_else(|| left.len().min(right.len()))
}

fn mismatch_window(source: &str, other: &str) -> String {
    let diff = first_diff(source, other);
    let start = source[..diff]
        .char_indices()
        .rev()
        .nth(80)
        .map_or(0, |(index, _)| index);
    let end = source[diff..]
        .char_indices()
        .nth(180)
        .map_or(source.len(), |(index, _)| diff + index);
    preview(&source[start..end])
}

fn assert_empty(label: &str, values: &[String]) {
    assert!(values.is_empty(), "{label}:\n{}", values.join("\n"));
}

fn assert_clean_corpus(report: &Report) {
    assert!(
        report.unreadable_count == 0
            && report.old_error_skips == 0
            && report.s2_refusal_count == 0
            && report.divergence_count == 0,
        "corpus unreadable files ({}):\n{}\n\ncorpus old-lane error skips ({}):\n{}\n\ncorpus S2 refusals ({}) by reason {:?}:\n{}\n\ncorpus divergences ({}):\n{}",
        report.unreadable_count,
        report.unreadable.join("\n"),
        report.old_error_skips,
        report.old_error_samples.join("\n"),
        report.s2_refusal_count,
        report.s2_refusal_reasons,
        report.s2_refusals.join("\n"),
        report.divergence_count,
        report.divergences.join("\n"),
    );
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
    ] {
        compare_sfc_template(name, source, &mut report);
    }

    assert_eq!(report.templates, 2);
    assert_eq!(
        report.old_error_skips, 0,
        "nested interactive-content recoveries should reach the DOM comparison lane: {:?}",
        report.old_error_samples
    );
    assert_eq!(
        report.s2_refusal_count, 0,
        "S2 should emit so any remaining mismatch is counted as a divergence: {:?}",
        report.s2_refusals
    );
    assert_eq!(report.compared, 2);
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
    assert_eq!(
        report.old_error_codes,
        vec![ErrorCode::InvalidEndTag],
        "the hard invalid end tag must remain visible in skip evidence: {:?}",
        report.old_error_samples
    );
}
