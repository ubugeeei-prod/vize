use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};

use criterion::{Criterion, criterion_group, criterion_main};
use vize::artifact_graph::{VizeGraphConfig, analysis_roots, compiler_roots, create_compilation};
use vize_atlas::ProductId;

struct TrackingAllocator;

static LIVE_BYTES: AtomicUsize = AtomicUsize::new(0);
static PEAK_BYTES: AtomicUsize = AtomicUsize::new(0);
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

#[global_allocator]
static ALLOCATOR: TrackingAllocator = TrackingAllocator;

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            record_allocation(layout.size());
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        LIVE_BYTES.fetch_sub(layout.size(), Ordering::Relaxed);
        unsafe { System.dealloc(pointer, layout) };
    }

    unsafe fn realloc(&self, pointer: *mut u8, old: Layout, new_size: usize) -> *mut u8 {
        let replacement = unsafe { System.realloc(pointer, old, new_size) };
        if !replacement.is_null() {
            if new_size >= old.size() {
                record_allocation(new_size - old.size());
            } else {
                LIVE_BYTES.fetch_sub(old.size() - new_size, Ordering::Relaxed);
            }
        }
        replacement
    }
}

fn record_allocation(bytes: usize) {
    ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
    let live = LIVE_BYTES.fetch_add(bytes, Ordering::Relaxed) + bytes;
    PEAK_BYTES.fetch_max(live, Ordering::Relaxed);
}

const SOURCE: &str = r#"<script setup lang="ts">
const ready = true
const items = [{ id: 1, label: 'one' }]
</script>
<template>
  <section v-if="ready">
    <p v-for="item in items" :key="item.id">{{ item.label }}</p>
  </section>
</template>"#;

#[derive(Debug, Clone, Copy)]
struct GraphCost {
    queries: u64,
    executions: u64,
    cache_entries: usize,
}

#[derive(Debug, Clone, Copy)]
struct AllocationCost {
    allocations: usize,
    peak_bytes: usize,
}

fn run_case(roots: &[ProductId]) -> GraphCost {
    let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
    let source = compilation.add_source("Benchmark.vue", SOURCE).unwrap();
    let plan = compilation.plan(source, roots.iter().copied()).unwrap();
    let outcome = compilation.execute(plan).unwrap();
    let (queries, executions) =
        outcome
            .plan()
            .products()
            .iter()
            .fold((0, 0), |(queries, executions), product| {
                let counters = compilation.counters().for_id(*product);
                (
                    queries + counters.queries(),
                    executions + counters.executions(),
                )
            });
    GraphCost {
        queries,
        executions,
        cache_entries: compilation.cache().len(),
    }
}

fn allocation_cost(roots: &[ProductId]) -> (GraphCost, AllocationCost) {
    let baseline_live = LIVE_BYTES.load(Ordering::Relaxed);
    let baseline_allocations = ALLOCATIONS.load(Ordering::Relaxed);
    PEAK_BYTES.store(baseline_live, Ordering::Relaxed);
    let cost = black_box(run_case(roots));
    let peak = PEAK_BYTES
        .load(Ordering::Relaxed)
        .saturating_sub(baseline_live);
    let allocations = ALLOCATIONS
        .load(Ordering::Relaxed)
        .saturating_sub(baseline_allocations);
    (
        cost,
        AllocationCost {
            allocations,
            peak_bytes: peak,
        },
    )
}

fn benchmark_artifact_graph(criterion: &mut Criterion) {
    let cases = [
        ("compiler_only", compiler_roots(true, false, false)),
        ("lint_only", analysis_roots(true, false)),
        ("typecheck_only", analysis_roots(false, true)),
        ("combined", analysis_roots(true, true)),
    ];
    let mut group = criterion.benchmark_group("artifact_graph");
    for (name, roots) in cases {
        let (graph, allocation) = allocation_cost(&roots);
        println!(
            "artifact_graph_baseline case={name} allocations={} peak_bytes={} queries={} executions={} cache_entries={}",
            allocation.allocations,
            allocation.peak_bytes,
            graph.queries,
            graph.executions,
            graph.cache_entries,
        );
        group.bench_function(name, |bencher| {
            bencher.iter(|| black_box(run_case(black_box(&roots))));
        });
    }
    group.finish();
}

criterion_group!(benches, benchmark_artifact_graph);
criterion_main!(benches);
