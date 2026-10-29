//! One fresh source generation, with full packets and explicit profiling overlays.
#![expect(
    clippy::disallowed_macros,
    reason = "public fixtures use standard formatting"
)]

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

fn authored_source(count: usize, script_bindings: usize, shape: &str) -> String {
    let mut source = String::from("<!-- 日本 🧭 -->\r\n<script setup lang=\"ts\">\r\n");
    for index in 0..script_bindings {
        source.push_str(&format!(
            "const choiceItem{index} = {{id: {index}, label: '日本 🧭'}};\r\n"
        ));
    }
    source.push_str("const { label: localLabel, ...rest } = defineProps<{label: string; class?: string}>();\r\n");
    source.push_str("</script>\r\n<template><section :class=\"rest.class\">{{ localLabel }}");
    for index in 0..count {
        match shape {
            "camel" => source.push_str(&format!("<choice-item-{index} />")),
            "missing" => source.push_str(&format!("<missing-item-{index} />")),
            "dynamic" => source.push_str(&format!("<component :is=\"choiceItem{index}\" />")),
            "builtin" => source.push_str("<KeepAlive><span /></KeepAlive>"),
            "global" => source.push_str("<GlobalWidget :value=\"1\" />"),
            _ => assert_eq!(shape, "empty", "unsupported public fixture shape"),
        }
    }
    source.push_str("</section></template>\r\n");
    source
}

#[test]
#[ignore = "dedicated source profiling workflow supplies input, output, and common overlays"]
fn component_scope_binding_generation_profile() {
    let count = std::env::var("VIZE_COMPONENT_PROFILE_COUNT")
        .map(|value| value.parse::<usize>().unwrap())
        .unwrap_or(32);
    assert!(matches!(count, 0 | 32 | 128 | 512));
    let script_bindings = std::env::var("VIZE_COMPONENT_PROFILE_BINDINGS")
        .map(|value| value.parse::<usize>().unwrap())
        .unwrap_or(count);
    assert!(matches!(script_bindings, 32 | 128 | 512 | 10000));
    let mode = std::env::var("VIZE_COMPONENT_PROFILE_MODE").unwrap_or_else(|_| "allocation".into());
    assert!(matches!(mode.as_str(), "allocation" | "runtime-off"));
    let shape = std::env::var("VIZE_COMPONENT_PROFILE_SHAPE").unwrap_or_else(|_| "camel".into());
    assert!(matches!(
        shape.as_str(),
        "camel" | "missing" | "dynamic" | "empty" | "builtin" | "global"
    ));
    let file = std::env::var("VIZE_COMPONENT_PROFILE_FILE").expect("stable identical input path");
    let source = authored_source(count, script_bindings, &shape);
    let profiler = global_profiler();
    profiler.disable();
    profiler.clear();
    reset_allocation_counters();
    if mode == "allocation" {
        profiler.enable();
    }
    let started = Instant::now();
    let packet = generate_vue_content_mapper_transform(Path::new(&file), &source).unwrap();
    let elapsed = started.elapsed();
    profiler.disable();
    let allocation = allocation_snapshot();
    let mut spans = profiler.span_snapshot().into_iter().map(|(name, _, metrics)| json!({
        "name": name, "calls": metrics.count, "nanoseconds": metrics.total_duration.as_nanos(),
        "allocationCalls": metrics.alloc_calls, "requestedBytes": metrics.alloc_bytes,
        "selfAllocationCalls": metrics.self_alloc_calls, "selfRequestedBytes": metrics.self_alloc_bytes,
    })).collect::<Vec<_>>();
    spans.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    if mode == "allocation" {
        assert!(allocation.allocation_calls() > 0);
        assert_eq!(allocation.allocation_failures(), 0);
        assert!(
            spans
                .iter()
                .any(|span| span["name"] == "canon.component_binding_observer.with_bindings")
        );
    } else {
        assert_eq!(allocation.allocation_calls(), 0);
        assert!(spans.is_empty());
    }
    let observation = json!({
        "schema": "vize.component-scope-binding-profile.v1",
        "sourceRevision": std::env::var("VIZE_COMPONENT_PROFILE_SOURCE").unwrap(),
        "shape":shape,"count":count,"scriptBindings":script_bindings,"mode":mode,"file":file,"source":source,"packet":packet,
        "elapsedNanoseconds":elapsed.as_nanos(),
        "allocation":{"calls":allocation.allocation_calls(),"requestedBytes":allocation.requested_bytes(),
            "releasedBytes":allocation.released_bytes(),"failures":allocation.allocation_failures()},
        "spans":spans,
        "scope":"one fresh source generation; fixture creation and serialization excluded; explicit common profiling-only with_bindings overlay; runtime-off uses profiling build, not normal release",
        "cold":{"generationCalls":1,"warmups":0,"persistentCache":false}
    });
    let output = std::env::var_os("VIZE_COMPONENT_PROFILE_OUTPUT").expect("raw packet output");
    std::fs::write(output, serde_json::to_vec_pretty(&observation).unwrap()).unwrap();
}
