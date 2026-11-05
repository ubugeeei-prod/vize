//! One fresh mapper call; allocation and runtime-off windows use the same profiling binary.
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

fn source(family: &str, count: usize) -> String {
    if family == "refusal" {
        return format!(
            "{}<template><div></template>",
            "<!-- 日本 🧭 -->\r\n".repeat(count)
        )
        .into();
    }
    let mut text = String::from(
        "<!-- 日本 🧭 -->\r\n<script setup lang=\"ts\">\r\nimport Child from './Child.vue';\r\nimport { useTemplateRef } from 'vue';\r\nconst label = '日本 🧭';\r\nconst values = [{ id: 1, label }];\r\nconst pick = (value: unknown) => value;\r\nconst child = useTemplateRef('child');\r\n",
    );
    if family == "script-only" {
        for index in 0..count {
            text.push_str(&format!(
                "const value{index} = {{ id: {index}, label }};\r\n"
            ));
        }
        text.push_str("</script>\r\n");
        return text;
    }
    text.push_str("</script>\r\n<template><main>\r\n");
    for index in 0..count {
        match family {
            "native" => text.push_str(&format!("<button :title=\"label\" @click=\"pick({index})\">{{{{ label }}}} {{{{ values[0].id }}}}</button>\r\n")),
            "components" => text.push_str(&format!("<Child :item=\"values[0]\" :count=\"{index}\" @select=\"pick\" @update:value=\"value => pick(value)\" />\r\n")),
            "loop-if" => text.push_str("<section v-for=\"(value, i) in values\" :key=\"i\"><div v-if=\"value.id\">{{ value.label }}</div><span v-else>{{ i }}</span></section>\r\n"),
            "slot-ref" => text.push_str("<Child ref=\"child\"><template #default=\"{ item }\"><span>{{ item.id }}</span></template></Child>\r\n"),
            _ => panic!("unknown fixture family"),
        }
    }
    text.push_str("</main></template>\r\n");
    text
}

#[test]
#[ignore = "dedicated workflow supplies a fresh process and evidence destination"]
fn complete_content_mapper_span_profile() {
    let family = std::env::var("VIZE_SPAN_FAMILY").unwrap();
    let count = std::env::var("VIZE_SPAN_COUNT")
        .unwrap()
        .parse::<usize>()
        .unwrap();
    assert!(matches!(count, 32 | 256));
    let mode = std::env::var("VIZE_SPAN_MODE").unwrap();
    assert!(matches!(mode.as_str(), "runtime-off" | "allocation"));
    let input = source(&family, count);
    let profiler = global_profiler();
    profiler.disable();
    profiler.clear();
    reset_allocation_counters();
    if mode == "allocation" {
        profiler.enable();
    }
    let started = Instant::now();
    let packet = generate_vue_content_mapper_transform(Path::new("src/Span.vue"), &input).unwrap();
    let elapsed = started.elapsed();
    profiler.disable();
    let allocation = allocation_snapshot();
    assert_eq!(allocation.allocation_failures(), 0);
    if mode == "runtime-off" {
        assert_eq!(allocation.allocation_calls(), 0);
        assert_eq!(profiler.span_snapshot().len(), 0);
    }
    // Whole serializer output is retained as a string: field omission and order remain observable.
    let packet = serde_json::to_string(&packet).unwrap();
    let observation = json!({
        "schema": "vize.content-mapper-span-profile.v1", "family": family, "count": count, "mode": mode,
        "sourceRevision": std::env::var("VIZE_SPAN_SOURCE").unwrap(), "source": input, "packet": packet,
        "elapsedNanoseconds": elapsed.as_nanos(),
        "allocation": {"calls": allocation.allocation_calls(), "requestedBytes": allocation.requested_bytes(),
            "releasedBytes": allocation.released_bytes(), "failures": allocation.allocation_failures()},
        "cold": {"generationCalls": 1, "warmups": 0, "persistentCache": false},
        "scope": "profiling binary, one fresh mapper call; fixture creation and packet serialization excluded from generation and allocation windows; runtime-off is not a normal release timing"
    });
    std::fs::write(
        std::env::var_os("VIZE_SPAN_OUTPUT").unwrap(),
        serde_json::to_vec(&observation).unwrap(),
    )
    .unwrap();
}
