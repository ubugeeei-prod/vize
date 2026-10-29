//! Fresh-process source generation and allocation evidence, with full packets.
#![expect(clippy::disallowed_macros, reason = "fixtures use standard formatting")]

use serde_json::json;
use std::{alloc::System, path::Path, time::Instant};
use vize_canon::generate_vue_content_mapper_transform;
use vize_l0::{
    String,
    profiler::{
        ProfilingAllocator, allocation_snapshot, global_profiler, reset_allocation_counters,
    },
};

#[global_allocator]
static ALLOCATOR: ProfilingAllocator<System> = ProfilingAllocator::from_allocator(System);

fn authored_source(count: usize, imports: bool) -> String {
    let mut source = String::from("<!-- 日本 🧭 -->\r\n<script setup lang=\"ts\">\r\n");
    if imports {
        source.push_str("import { ");
        for index in 0..count {
            if index != 0 {
                source.push_str(", ");
            }
            source.push_str(&format!("item{index}"));
        }
        source.push_str(" } from './model';\r\n");
    } else {
        for index in 0..count {
            source.push_str(&format!(
                "const item{index}: {{id: number; label: string}} = {{id: {index}, label: '日本 🧭'}};\r\n"
            ));
        }
    }
    source.push_str("const { label: localLabel, ...rest } = defineProps<{label: string; 'aria-label'?: string; class?: string}>();\r\n");
    source
        .push_str("</script>\r\n<template><section :class=\"rest.class\">{{ localLabel }} {{ item");
    let last = count - 1;
    source.push_str(vize_l0::cstr!("{last}").as_str());
    source.push_str(".id }}</section></template>\r\n");
    source
}

#[test]
fn complete_binding_anchor_generation_profile() {
    let count = std::env::var("VIZE_BINDING_PROFILE_COUNT")
        .map(|value| value.parse::<usize>().unwrap())
        .unwrap_or(32);
    assert!(matches!(count, 32 | 128 | 512));
    let mode = std::env::var("VIZE_BINDING_PROFILE_MODE").unwrap_or_else(|_| "allocation".into());
    assert!(matches!(mode.as_str(), "allocation" | "runtime-off"));
    let shape = std::env::var("VIZE_BINDING_PROFILE_SHAPE").unwrap_or_else(|_| "locals".into());
    assert!(matches!(shape.as_str(), "locals" | "imports"));
    let source = authored_source(count, shape == "imports");
    let profiler = global_profiler();
    profiler.disable();
    profiler.clear();
    reset_allocation_counters();
    if mode == "allocation" {
        profiler.enable();
    }
    let started = Instant::now();
    let packet =
        generate_vue_content_mapper_transform(Path::new("src/Bindings.vue"), &source).unwrap();
    let elapsed = started.elapsed();
    profiler.disable();
    let allocation = allocation_snapshot();
    let mut spans = profiler
        .span_snapshot()
        .into_iter()
        .map(|(name, _, metrics)| {
            json!({
                "name": name,
                "calls": metrics.count,
                "nanoseconds": metrics.total_duration.as_nanos(),
                "allocationCalls": metrics.alloc_calls,
                "requestedBytes": metrics.alloc_bytes,
                "selfAllocationCalls": metrics.self_alloc_calls,
                "selfRequestedBytes": metrics.self_alloc_bytes,
            })
        })
        .collect::<Vec<_>>();
    spans.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    if mode == "allocation" {
        let anchor = spans
            .iter()
            .find(|span| span["name"] == "canon.virtual_ts.emit_setup_binding_anchors")
            .expect("the real anchor emission must execute");
        assert_eq!(anchor["calls"], 1);
        assert!(allocation.allocation_calls() > 0);
        assert_eq!(allocation.allocation_failures(), 0);
    } else {
        assert_eq!(allocation.allocation_calls(), 0);
        assert_eq!(spans, Vec::<serde_json::Value>::new());
    }
    let observation = json!({
        "schema": "vize.setup-binding-anchor-profile",
        "sourceRevision": std::env::var("VIZE_BINDING_PROFILE_SOURCE").ok(),
        "shape": shape, "count": count, "mode": mode,
        "source": source, "packet": packet,
        "elapsedNanoseconds": elapsed.as_nanos(),
        "allocation": {"calls": allocation.allocation_calls(), "requestedBytes": allocation.requested_bytes(),
            "releasedBytes": allocation.released_bytes(), "failures": allocation.allocation_failures()},
        "spans": spans,
        "scope": "one source generation in a fresh process; fixture construction and serialization excluded; profiling binary runtime-off is not a normal release artifact timing",
        "cold": {"generationCalls": 1, "warmups": 0, "persistentCache": false},
    });
    if let Some(output) = std::env::var_os("VIZE_BINDING_PROFILE_OUTPUT") {
        std::fs::write(output, serde_json::to_vec_pretty(&observation).unwrap()).unwrap();
    }
}
