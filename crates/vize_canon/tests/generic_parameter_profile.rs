//! Paired source-API allocation receipts, separate from native engine timing.

#![expect(clippy::unwrap_used, reason = "qualification fails closed")]

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::hint::black_box;
use std::path::Path;
use std::time::Instant;

use serde_json::{Value, json};
use vize_canon::sfc_typecheck::{SfcTypeCheckOptions, type_check_sfc};
use vize_canon::{
    ContentMapperTransformOptions, generate_vue_content_mapper_transform_with_options,
};

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
struct Counts {
    calls: usize,
    requested: usize,
}

thread_local! {
    static COUNTS: Cell<Option<Counts>> = const { Cell::new(None) };
}

struct CountedSystem;

// The observer delegates allocation/deallocation unchanged to System. Only
// this thread's successful requests inside the measured interval are counted.
unsafe impl GlobalAlloc for CountedSystem {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            record(layout.size());
        }
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            record(layout.size());
        }
        pointer
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let pointer = unsafe { System.realloc(pointer, layout, size) };
        if !pointer.is_null() {
            record(size);
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) };
    }
}

fn record(size: usize) {
    let _ = COUNTS.try_with(|counts| {
        if let Some(mut current) = counts.get() {
            current.calls += 1;
            current.requested += size;
            counts.set(Some(current));
        }
    });
}

#[global_allocator]
static ALLOCATOR: CountedSystem = CountedSystem;

const ORDINARY: &str = "<script setup lang=\"ts\" generic=\"const T extends { id: number; label: string }\">\nconst props = defineProps<{ value: T }>()\ndefineEmits<{ change: [value: T] }>()\ndefineSlots<{ default: (props: { item: T }) => unknown }>()\n</script>\n<template>{{ props.value.label }}</template>";
const LITERAL: &str = "<script setup lang=\"ts\" generic=\"const T extends 'a,b' = 'a,b', U extends `x,${string}` = 'x,y', F extends (left: T, right: U) => readonly [T, U] = (left: T, right: U) => readonly [T, U]\">\ndefineProps<{ value: T; templateValue: U; callback?: F }>()\n</script>\n<template><div /></template>";
const PLAIN: &str = "<script setup lang=\"ts\">const props = defineProps<{ value: string }>()</script><template>{{ props.value }}</template>";

fn packet(source: &str, mapper: bool) -> Value {
    if mapper {
        serde_json::to_value(
            generate_vue_content_mapper_transform_with_options(
                Path::new("src/Profile.vue"),
                source,
                ContentMapperTransformOptions::default(),
            )
            .unwrap(),
        )
        .unwrap()
    } else {
        let options = SfcTypeCheckOptions::new("src/Profile.vue").with_virtual_ts();
        let result = type_check_sfc(source, &options);
        let mut packet = serde_json::to_value(result).unwrap();
        packet.as_object_mut().unwrap().remove("analysis_time_ms");
        packet
    }
}

fn operation(source: &str, mapper: bool) {
    if mapper {
        black_box(
            generate_vue_content_mapper_transform_with_options(
                Path::new("src/Profile.vue"),
                black_box(source),
                ContentMapperTransformOptions::default(),
            )
            .unwrap(),
        );
    } else {
        let options = SfcTypeCheckOptions::new("src/Profile.vue").with_virtual_ts();
        black_box(type_check_sfc(black_box(source), &options));
    }
}

#[test]
#[ignore = "paired source allocation/timing qualification; run explicitly on base and head"]
fn complete_source_generic_parameter_profile() {
    // Both source executables observe the same filesystem context. Differing
    // checkout path lengths must not masquerade as allocation improvements.
    let working_directory = std::env::var_os("VIZE_GENERIC_PARAMETER_PROFILE_CWD").unwrap();
    std::fs::create_dir_all(&working_directory).unwrap();
    std::env::set_current_dir(&working_directory).unwrap();
    let mut rows = Vec::new();
    for (name, source) in [
        ("ordinary", ORDINARY),
        ("literal", LITERAL),
        ("plain", PLAIN),
    ] {
        for mapper in [false, true] {
            for _ in 0..8 {
                operation(source, mapper);
            }
            let mut samples = Vec::new();
            let mut allocations = None;
            for _ in 0..3 {
                COUNTS.with(|counts| counts.set(Some(Counts::default())));
                let start = Instant::now();
                for _ in 0..100 {
                    operation(source, mapper);
                }
                let elapsed = start.elapsed().as_nanos();
                let current = COUNTS.with(|counts| counts.replace(None)).unwrap();
                if let Some(expected) = allocations {
                    assert_eq!(current, expected);
                }
                allocations = Some(current);
                samples.push(elapsed);
            }
            let counts = allocations.unwrap();
            rows.push(json!({"case": name, "mapper": mapper, "source": source,
                "iterationsPerSample": 100, "samplesNanoseconds": samples,
                "allocationCallsPerSample": counts.calls, "requestedBytesPerSample": counts.requested,
                "packet": packet(source, mapper)}));
        }
    }
    assert_eq!(rows.len(), 6);
    let capture = std::env::var_os("VIZE_GENERIC_PARAMETER_PROFILE_CAPTURE").unwrap();
    std::fs::write(capture, serde_json::to_vec_pretty(&json!({
        "scope": "Current-thread source APIs only; no native provider, end-to-end CLI, or instruction ceiling claim",
        "allocationDefinition": "Successful System alloc/alloc_zeroed/realloc requests and requested bytes, excluding warmup and observation serialization",
        "workingDirectory": std::env::current_dir().unwrap(),
        "rows": rows,
    })).unwrap()).unwrap();
}
