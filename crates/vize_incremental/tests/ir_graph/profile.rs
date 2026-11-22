//! Independent A/B and B/A medians. Both requests produce exactly the same
//! compiler/effect rows, editor spans and independently verified witness.
use davinci_harness::alloc::{self, CountingAllocator};
use std::{hint::black_box, time::Instant};
use vize_l0::{Allocator, Lang, Span};
use vize_l3::{
    op::{EffectId, OpId, RegionId},
    query::{Direction, EdgeKinds, OpGraph, ReachQuery, ReachStep},
};

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator::mimalloc();

#[derive(Debug, PartialEq, Eq)]
struct Products {
    effects: Vec<(OpId, RegionId, Option<EffectId>)>,
    spans: Vec<(OpId, Span)>,
    steps: Vec<ReachStep>,
}

fn products(graph: &OpGraph<'_>, query: ReachQuery, joined: bool) -> Products {
    let mut effects = Vec::new();
    let mut spans = Vec::new();
    let steps = if joined {
        graph
            .reach_with(query, |op| {
                effects.push((op.id, op.region, op.effect));
                spans.push((op.id, op.span));
            })
            .unwrap()
    } else {
        let first = graph
            .reach_with(query, |op| effects.push((op.id, op.region, op.effect)))
            .unwrap();
        graph.check(query, &first).unwrap();
        graph
            .reach_with(query, |op| spans.push((op.id, op.span)))
            .unwrap()
    };
    graph.check(query, &steps).unwrap();
    Products {
        effects,
        spans,
        steps,
    }
}

fn profile(source: &str, label: &str) {
    let arena = Allocator::default();
    let native = vize_l1_to_l2::native::lower_component_native(&arena, source, Lang::Js).unwrap();
    assert!(native.is_supported());
    let lower = vize_l2_to_l3::lower(&arena, native.artifact.root());
    let ir = lower.l3ir();
    let graph = OpGraph::new(ir).unwrap();
    let query = ReachQuery {
        start: ir.ops.first().unwrap().id,
        direction: Direction::Forward,
        kinds: EdgeKinds::ALL,
    };
    for cold in [false, true] {
        let read = |joined| {
            if cold {
                products(&OpGraph::new(black_box(ir)).unwrap(), query, joined)
            } else {
                products(black_box(&graph), query, joined)
            }
        };
        let (before, a) = alloc::measure_returning(|| read(false));
        let (after, b) = alloc::measure_returning(|| read(true));
        let (a, b) = (a.unwrap(), b.unwrap());
        assert_eq!(before, after);
        assert!(b.calls < a.calls);
        assert!(b.peak_bytes_over_start <= a.peak_bytes_over_start);
        let sample = |joined| {
            let start = Instant::now();
            for _ in 0..64 {
                black_box(read(joined));
            }
            start.elapsed().as_nanos() / 64
        };
        let mut old = Vec::new();
        let mut new = Vec::new();
        for row in 0..35 {
            let (a, b) = if row % 2 == 0 {
                (sample(false), sample(true))
            } else {
                let b = sample(true);
                (sample(false), b)
            };
            if row >= 4 {
                old.push(a);
                new.push(b);
            }
        }
        old.sort_unstable();
        new.sort_unstable();
        let (before_ns, after_ns) = (old[old.len() / 2], new[new.len() / 2]);
        println!(
            "graph source={label} ops={} edges={} reached={} cold={cold} ns={before_ns}/{after_ns} calls={}/{} peak={}/{}",
            ir.ops.len(),
            ir.edges.len(),
            after.steps.len(),
            a.calls,
            b.calls,
            a.peak_bytes_over_start,
            b.peak_bytes_over_start
        );
        assert!(
            after_ns <= before_ns,
            "{label} cold={cold}: {before_ns} -> {after_ns}"
        );
    }
}

fn main() {
    alloc::mark_installed();
    profile("<div/>", "one");
    profile(
        "<input :title=\"label\"><p>{{label}}</p><span>字</span>",
        "siblings",
    );
    let source = "<p :title=\"title\">{{title}}</p>".repeat(64);
    profile(&source, "64-siblings");
}
