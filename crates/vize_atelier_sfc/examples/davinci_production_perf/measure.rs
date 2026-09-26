//! Admission and output observations are separate from profile-disabled timing.

use super::{
    corpus::{Input, hash},
    shapes::{Shape, compile, explicit_vapor_source},
};
use serde_json::{Value, json};
use std::{hint::black_box, time::Instant};
use vize_atelier_sfc::{SfcCompileResult, SfcError, SfcParseOptions, parse_sfc};
use vize_s0::{
    String, cstr,
    profiler::{CounterSummary, ProfileExportOptions, global_profiler},
};

fn compile_source(input: &Input, shape: Shape) -> Result<SfcCompileResult, SfcError> {
    let descriptor = parse_sfc(
        &input.source,
        SfcParseOptions {
            filename: input.filename.clone(),
            ..Default::default()
        },
    )?;
    compile(&descriptor, &input.filename, shape)
}

fn retained<R>(shape: Shape, f: impl FnOnce() -> R) -> R {
    let vapor = || vize_atelier_vapor::compile::benchmark::with_retained_lane(f);
    match shape {
        Shape::DomInline | Shape::DomModule => {
            vize_atelier_dom::differential::with_legacy_lane(vapor)
        }
        Shape::Ssr => vize_atelier_ssr::differential::with_legacy_lane(vapor),
        Shape::Vapor => vapor(),
    }
}

fn lane(counters: &CounterSummary, backend: Shape) -> std::io::Result<String> {
    let mut selections = Vec::new();
    for entry in &counters.entries {
        let selection = [Shape::DomInline, Shape::Ssr, Shape::Vapor]
            .iter()
            .any(|shape| {
                entry
                    .name
                    .strip_prefix(shape.namespace())
                    .is_some_and(|suffix| {
                        suffix == "accepted"
                            || suffix == "rejected"
                            || suffix.starts_with("legacy.")
                    })
            });
        if selection && !entry.name.starts_with(backend.namespace()) && entry.total > 0 {
            return Err(std::io::Error::other(cstr!(
                "unexpected backend selection {} for {}",
                entry.name,
                backend.id()
            )));
        }
        let Some(suffix) = entry.name.strip_prefix(backend.namespace()) else {
            continue;
        };
        if suffix == "accepted" || suffix == "rejected" || suffix.starts_with("legacy.") {
            for _ in 0..entry.total {
                selections.push(String::from(suffix));
            }
        }
    }
    match selections.len() {
        0 => Ok("unrecorded".into()),
        1 => Ok(selections.remove(0)),
        count => Err(std::io::Error::other(cstr!(
            "{} selection-law violation: {count}",
            backend.id()
        ))),
    }
}

fn messages(result: &Result<SfcCompileResult, SfcError>) -> Value {
    match result {
        Ok(result) => json!({
            "errors": result.errors.iter().map(|e| &e.message).collect::<Vec<_>>(),
            "warnings": result.warnings.iter().map(|e| &e.message).collect::<Vec<_>>(),
        }),
        Err(error) => json!({"error": error}),
    }
}

fn fingerprint(result: &Result<SfcCompileResult, SfcError>) -> Value {
    match result {
        Ok(result) => json!({
            "code_sha256": hash(result.code.as_bytes()),
            "css_sha256": result.css.as_ref().map(|css| hash(css.as_bytes())),
            "errors": result.errors, "warnings": result.warnings,
            "map_present": result.map.is_some(),
        }),
        Err(error) => json!({"error": error}),
    }
}

fn selector_probe(shape: Shape) -> Result<Value, Box<dyn std::error::Error>> {
    let input = Input {
        filename: "SelectorProbe.vue".into(),
        source: "<template><main>probe</main></template>".into(),
        sha256: hash(b"<template><main>probe</main></template>"),
    };
    let profiler = global_profiler();
    profiler.clear();
    profiler.enable();
    let selected = compile_source(&input, shape)
        .map_err(|error| std::io::Error::other(cstr!("selector probe: {}", error.message)))?;
    let selected_lane = lane(&profiler.counter_summary(), shape)?;
    profiler.clear();
    let legacy = retained(shape, || compile_source(&input, shape)).map_err(|error| {
        std::io::Error::other(cstr!("retained selector probe: {}", error.message))
    })?;
    let retained_lane = lane(&profiler.counter_summary(), shape)?;
    profiler.disable();
    profiler.clear();
    let expected = match shape {
        Shape::DomInline | Shape::DomModule => "legacy.forced",
        Shape::Ssr => "unrecorded",
        Shape::Vapor => "legacy.selected",
    };
    if selected_lane != "accepted" || retained_lane != expected {
        return Err(std::io::Error::other(cstr!(
            "selector probe failed for {}: {selected_lane}/{retained_lane}, expected accepted/{expected}",
            shape.id()
        )).into());
    }
    Ok(json!({
        "source": input.source, "selected_lane": selected_lane, "retained_lane": retained_lane,
        "code_equal": selected.code == legacy.code,
    }))
}

fn timed(inputs: &[&Input], shape: Shape) -> u64 {
    let start = Instant::now();
    for _ in 0..5 {
        for input in inputs {
            drop(black_box(compile_source(black_box(input), shape)));
        }
    }
    // A bounded fixture batch cannot overflow u64 nanoseconds.
    u64::try_from(start.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

fn samples(inputs: &[&Input], shape: Shape) -> Value {
    if inputs.is_empty() {
        return json!({"files": 0});
    }
    black_box(timed(inputs, shape));
    black_box(retained(shape, || timed(inputs, shape)));
    let mut selected = Vec::new();
    let mut legacy = Vec::new();
    for sample in 0..9 {
        if sample % 2 == 0 {
            selected.push(timed(inputs, shape));
            legacy.push(retained(shape, || timed(inputs, shape)));
        } else {
            legacy.push(retained(shape, || timed(inputs, shape)));
            selected.push(timed(inputs, shape));
        }
    }
    let mut selected_sorted = selected.clone();
    let mut legacy_sorted = legacy.clone();
    selected_sorted.sort_unstable();
    legacy_sorted.sort_unstable();
    let selected_median = selected_sorted.get(4).copied().unwrap_or(0);
    let legacy_median = legacy_sorted.get(4).copied().unwrap_or(0);
    json!({
        "files": inputs.len(), "passes_per_sample": 5,
        "selected_batch_ns": selected, "retained_batch_ns": legacy,
        "selected_median_batch_ns": selected_median,
        "retained_median_batch_ns": legacy_median,
        "selected_over_retained": selected_median as f64 / legacy_median as f64,
    })
}

fn profile(inputs: &[Input], shape: Shape, force_retained: bool) -> Value {
    let profiler = global_profiler();
    profiler.clear();
    profiler.enable();
    let batch = || {
        for input in inputs {
            drop(black_box(compile_source(input, shape)));
        }
    };
    if force_retained {
        retained(shape, batch);
    } else {
        batch();
    }
    profiler.disable();
    let report = profiler.export_report(&ProfileExportOptions {
        command: "davinci-production-perf-attribution",
        allocation: None,
        budget: Default::default(),
    });
    profiler.clear();
    json!(report)
}

pub fn attribution(inputs: &[Input], shape: Shape) -> Value {
    json!({
        "shape": shape.id(), "selected_profile": profile(inputs, shape, false),
        "retained_profile": profile(inputs, shape, true),
    })
}

pub fn run(inputs: &[Input], shape: Shape) -> Result<Value, Box<dyn std::error::Error>> {
    let selector_probe = selector_probe(shape)?;
    let profiler = global_profiler();
    let mut observations = Vec::new();
    let mut accepted = Vec::new();
    let mut fallback = Vec::new();
    let mut diagnostics = Vec::new();
    let mut no_template = Vec::new();
    let mut parse_errors = Vec::new();
    let mut routed_vapor = Vec::new();
    for input in inputs {
        let descriptor = parse_sfc(&input.source, SfcParseOptions::default());
        let has_template = descriptor.as_ref().is_ok_and(|d| d.template.is_some());
        let backend = if shape.is_dom() && descriptor.as_ref().is_ok_and(explicit_vapor_source) {
            Shape::Vapor
        } else {
            shape
        };
        profiler.clear();
        profiler.enable();
        let selected = compile_source(input, shape);
        let selected_lane = lane(&profiler.counter_summary(), backend)?;
        profiler.clear();
        let legacy = retained(shape, || compile_source(input, shape));
        let retained_lane = lane(&profiler.counter_summary(), backend)?;
        profiler.disable();
        profiler.clear();
        if retained_lane == "accepted" {
            return Err(std::io::Error::other(cstr!(
                "retained override did not pin {} {}",
                shape.id(),
                input.filename
            ))
            .into());
        }
        let messages_equal = messages(&selected) == messages(&legacy);
        let (code_equal, css_equal, map_equal) = match (&selected, &legacy) {
            (Ok(a), Ok(b)) => (a.code == b.code, a.css == b.css, a.map == b.map),
            _ => (false, false, false),
        };
        // Preserve the existing DOM whole-module acceptance oracle; Vapor
        // programs may differ and retain the independent TS-33 runtime gate.
        if selected_lane == "accepted"
            && backend.is_dom()
            && !(code_equal && css_equal && messages_equal)
        {
            return Err(std::io::Error::other(cstr!(
                "production DOM parity failed: {} {}",
                shape.id(),
                input.filename
            ))
            .into());
        }
        let clean = |result: &Result<SfcCompileResult, SfcError>| {
            result
                .as_ref()
                .is_ok_and(|result| result.errors.is_empty() && result.warnings.is_empty())
        };
        let cohort = if descriptor.is_err() {
            parse_errors.push(input);
            "parse_error"
        } else if !clean(&selected) || !clean(&legacy) {
            diagnostics.push(input);
            "diagnostic"
        } else if !has_template {
            no_template.push(input);
            "no_template"
        } else if backend != shape {
            routed_vapor.push(input);
            "routed_vapor"
        } else if selected_lane == "accepted" {
            accepted.push(input);
            "accepted"
        } else {
            fallback.push(input);
            "fallback"
        };
        observations.push(json!({
            "filename": input.filename, "source_sha256": input.sha256,
            "has_template": has_template, "compiled": selected.is_ok(),
            "retained_compiled": legacy.is_ok(),
            "bytes": input.source.len(), "backend": backend.id(),
            "selected_lane": selected_lane, "retained_lane": retained_lane, "cohort": cohort,
            "code_equal": code_equal, "css_equal": css_equal,
            "messages_equal": messages_equal, "map_equal": map_equal,
            "selected": fingerprint(&selected), "retained": fingerprint(&legacy),
        }));
    }
    profiler.disable();
    let all = inputs.iter().collect::<Vec<_>>();
    let cohorts = json!({
        "all": samples(&all, shape), "accepted": samples(&accepted, shape),
        "fallback": samples(&fallback, shape), "diagnostic": samples(&diagnostics, shape),
        "no_template": samples(&no_template, shape), "parse_error": samples(&parse_errors, shape),
        "routed_vapor": samples(&routed_vapor, shape),
    });
    Ok(json!({
        "shape": shape.id(), "selector_probe": selector_probe,
        "observations": observations, "cohorts": cohorts,
        "retained_selector": match shape {
            Shape::DomInline | Shape::DomModule => "DOM legacy.forced; explicit Vapor legacy.selected",
            Shape::Ssr => "SSR LegacyOnly scoped override; it does not emit a selection counter",
            Shape::Vapor => "Vapor legacy.selected scoped override",
        },
        "selected_profile": profile(inputs, shape, false),
        "retained_profile": profile(inputs, shape, true),
    }))
}
