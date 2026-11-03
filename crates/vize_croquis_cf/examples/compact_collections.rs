//! Identical single-thread workload for exact base/head collection measurements.
#![expect(clippy::expect_used, reason = "measurement failures abort like tests")]

use level_harness::alloc::{CountingAllocator, mark_installed, measure};
use serde_json::{Value, json};
use std::{hint::black_box, time::Instant};
use vize_croquis::setup_context::SetupContextTracker;
use vize_croquis::{Analyzer, AnalyzerOptions, ScopeChain, ScopeKind};
use vize_croquis_cf::{DependencyEdge, DependencyGraph, FileId, ModuleNode};

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator::mimalloc();

fn record<T>(name: &str, mut routine: impl FnMut() -> T) -> Value {
    for _ in 0..8 {
        black_box(routine());
    }
    let mut samples = Vec::new();
    for _ in 0..9 {
        let started = Instant::now();
        for _ in 0..64 {
            black_box(routine());
        }
        samples.push(started.elapsed().as_nanos() as u64 / 64);
    }
    let metrics = measure(|| black_box(routine())).expect("counting allocator installed");
    let mut sorted = samples.clone();
    sorted.sort_unstable();
    json!({"name": name, "samples_ns": samples, "p50_ns": sorted.get(4).expect("nine samples"),
        "allocations": metrics.calls, "peak_bytes": metrics.peak_bytes_over_start})
}

fn graph(count: u32, stride: u32, offset: u32) -> DependencyGraph {
    let mut graph = DependencyGraph::new();
    for index in 0..count {
        graph.add_node(ModuleNode::new(
            FileId::new(index * stride + offset),
            "test.vue",
        ));
    }
    // Independent eight-node rings keep recursion bounded while visiting every
    // node. The sparse case spans a large ID domain without deriving
    // a dense allocation from the largest external ID.
    for index in 0..count {
        let next = if index % 8 == 7 { index - 7 } else { index + 1 };
        graph.add_edge(
            FileId::new(index * stride + offset),
            FileId::new(next * stride + offset),
            DependencyEdge::Import,
        );
    }
    graph
}

fn analyze(source: &str, setup: bool) -> Vec<vize_croquis::Croquis> {
    (0..64)
        .map(|_| {
            let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
            if setup {
                analyzer.analyze_script_setup(source);
            } else {
                analyzer.analyze_script_plain(source);
            }
            analyzer.finish()
        })
        .collect()
}

fn main() {
    mark_installed();
    let mut cases = vec![record("empty_setup_trackers_10000", || {
        (0..10000)
            .map(|_| SetupContextTracker::new())
            .collect::<Vec<_>>()
    })];
    cases.push(record("actual_script_setup_clean_64", || {
        analyze("const count = 1", true)
    }));
    cases.push(record("actual_plain_script_violations_64", || {
        analyze(
            "import { ref } from 'vue'; export const state = ref(0)",
            false,
        )
    }));
    cases.push(record("scope_rows_1024", || {
        let mut scopes = ScopeChain::with_capacity(1024);
        for _ in 1..1024 {
            scopes.enter_scope(ScopeKind::Closure);
        }
        for id in scopes.iter().map(|scope| scope.id) {
            black_box(scopes.get_scope(id));
        }
        scopes
    }));
    let mut fingerprints = Vec::new();
    for (name, count, stride, offset) in [
        ("dense_cycles_1024", 1024, 1, 0),
        ("dense_cycles_4096", 4096, 1, 0),
        ("sparse_cycles_1024", 1024, 1048576, 0),
        ("shifted_cycles_4096", 4096, 1, 1 << 30),
    ] {
        let mut graph = graph(count, stride, offset);
        graph.detect_circular_dependencies();
        fingerprints.push(json!({"name": name, "cycles": graph.circular_dependencies().iter().map(|cycle| cycle.iter().map(|id| id.as_u32()).collect::<Vec<_>>()).collect::<Vec<_>>() }));
        cases.push(record(name, || {
            graph.detect_circular_dependencies();
            graph.circular_dependencies().len()
        }));
    }
    let mut semantic_fingerprints = Vec::new();
    for (source, setup, expected_violations) in [
        ("const count = 1", true, 0),
        (
            "import { ref } from 'vue'; export const state = ref(0)",
            false,
            1,
        ),
    ] {
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        if setup {
            analyzer.analyze_script_setup(source);
        } else {
            analyzer.analyze_script_plain(source);
        }
        let result = analyzer.finish();
        assert_eq!(result.setup_context.count(), expected_violations);
        semantic_fingerprints.push(json!({"snapshot": result.semantic_snapshot(),
            "setup_violations": result.setup_context.violations().iter().map(|v| (v.kind.to_display(), v.api_name.as_str(), v.start, v.end)).collect::<Vec<_>>() }));
    }
    let report = json!({"schema": 1, "architecture": std::env::consts::ARCH,
        "os": std::env::consts::OS, "setup_tracker_bytes": size_of::<SetupContextTracker>(),
        "scope_id_bytes": size_of::<vize_croquis::ScopeId>(), "cases": cases,
        "cycle_fingerprints": fingerprints, "semantic_fingerprints": semantic_fingerprints});
    let path = std::env::args().nth(1).expect("output path argument");
    std::fs::write(
        path,
        serde_json::to_vec_pretty(&report).expect("JSON encoding"),
    )
    .expect("report write");
}
