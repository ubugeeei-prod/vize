//! Actual collector calls and allocations, separately from uninstrumented time.

use serde_json::json;
use std::{hint::black_box, time::Instant};
use vize_atelier_sfc::collect_template_asset_urls;
use vize_l0::profiler::{ProfilingAllocator, allocation_snapshot, global_profiler};

#[global_allocator]
static ALLOCATOR: ProfilingAllocator<mimalloc::MiMalloc> =
    ProfilingAllocator::from_allocator(mimalloc::MiMalloc);

fn main() {
    let cases = [
        (
            "no-assets",
            "<script setup>const _imports_0 = 1</script><template><p>plain</p></template>",
        ),
        (
            "ordinary",
            "<script setup>const title = 'ordinary'</script><template><img src='./a.svg' :title='title'></template>",
        ),
        (
            "script-collision",
            "<script setup>const _imports_0 = 1</script><template><img src='./a.svg'></template>",
        ),
        (
            "script-negative",
            "<script>const label = '_imports_0'; export default {label}</script><template><img src='./a.svg'></template>",
        ),
        (
            "template-collision",
            "<template><img v-for='_imports_0 of items' src='./a.svg' :title='_imports_0'></template>",
        ),
        (
            "entities-negative",
            "<template><img src='./a.svg' title='A &amp; B'></template>",
        ),
    ];
    let profiler = global_profiler();
    let mut rows = Vec::new();
    for (name, source) in cases {
        for _ in 0..10 {
            drop(black_box(collect_template_asset_urls(
                source,
                None,
                Some("App.vue"),
            )));
        }
        let mut samples = Vec::new();
        for _ in 0..9 {
            let start = Instant::now();
            for _ in 0..100 {
                drop(black_box(collect_template_asset_urls(
                    source,
                    None,
                    Some("App.vue"),
                )));
            }
            samples.push(start.elapsed().as_nanos());
        }
        profiler.clear();
        profiler.enable();
        let assets = collect_template_asset_urls(source, None, Some("App.vue"));
        profiler.disable();
        let allocation = allocation_snapshot();
        let counters = profiler.counter_summary().entries.into_iter().map(|counter| {
            json!({"name": counter.name, "samples": counter.samples, "total": counter.total})
        }).collect::<Vec<_>>();
        rows.push(json!({
            "case": name, "source": source, "timing_calls_per_sample": 100,
            "timing_profiler_enabled": false, "timing_samples_ns": samples,
            "attribution_calls": 1, "attribution_allocation_calls": allocation.allocation_calls(),
            "attribution_alloc_bytes": allocation.alloc_bytes + allocation.alloc_zeroed_bytes + allocation.realloc_new_bytes,
            "counters": counters,
            "assets": assets.iter().map(|asset| json!({"url": asset.url, "varName": asset.var_name})).collect::<Vec<_>>(),
        }));
    }
    println!(
        "{}",
        json!({"source_sha": std::env::var("SOURCE_SHA").unwrap_or_default(), "scope": "collector-only; timing excludes detailed profiling; attribution is actual allocation requests, not retained memory; necessary correctness work is not a speedup claim", "cases": rows})
    );
}
