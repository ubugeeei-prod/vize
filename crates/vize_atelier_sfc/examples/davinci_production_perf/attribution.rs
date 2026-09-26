//! Diagnostic spans are collected separately for all and clean native inputs.

use super::{
    corpus::Input,
    measure::{compile_source, lane, retained},
    shapes::{Shape, explicit_vapor_source},
};
use serde_json::{Value, json};
use std::hint::black_box;
use vize_s0::profiler::{ProfileExportOptions, global_profiler};

pub(super) fn profile(inputs: &[&Input], shape: Shape, force_retained: bool) -> Value {
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

pub fn run(inputs: &[Input], shape: Shape) -> Result<Value, Box<dyn std::error::Error>> {
    let profiler = global_profiler();
    let all = inputs.iter().collect::<Vec<_>>();
    let mut accepted = Vec::new();
    let mut input_spans = Vec::new();
    for input in inputs {
        let descriptor = vize_atelier_sfc::parse_sfc(&input.source, Default::default());
        let backend = if shape.is_dom() && descriptor.as_ref().is_ok_and(explicit_vapor_source) {
            Shape::Vapor
        } else {
            shape
        };
        profiler.clear();
        profiler.enable();
        let selected = compile_source(input, shape);
        let selected_lane = lane(&profiler.counter_summary(), backend)?;
        let spans = profiler
            .summary()
            .entries
            .into_iter()
            .filter(|entry| entry.name.starts_with("atelier.vapor.bridge."))
            .map(|entry| {
                json!({
                    "key": entry.name, "count": entry.count,
                    "self_ns": entry.self_total.as_nanos(), "total_ns": entry.total.as_nanos(),
                })
            })
            .collect::<Vec<_>>();
        if !spans.is_empty() {
            input_spans
                .push(json!({"filename": input.filename, "lane": selected_lane, "spans": spans}));
        }
        profiler.disable();
        profiler.clear();
        let legacy = retained(shape, || compile_source(input, shape));
        let clean =
            |result: &Result<vize_atelier_sfc::SfcCompileResult, vize_atelier_sfc::SfcError>| {
                result
                    .as_ref()
                    .is_ok_and(|result| result.errors.is_empty() && result.warnings.is_empty())
            };
        if backend == shape && selected_lane == "accepted" && clean(&selected) && clean(&legacy) {
            accepted.push(input);
        }
    }
    Ok(json!({
        "shape": shape.id(), "accepted_files": accepted.len(), "input_bridge_spans": input_spans,
        "selected_profile": profile(&all, shape, false),
        "retained_profile": profile(&all, shape, true),
        "accepted_selected_profile": profile(&accepted, shape, false),
        "accepted_retained_profile": profile(&accepted, shape, true),
    }))
}
