//! Independent A/B and B/A medians. Both requests produce exactly the same
//! compiler/effect rows, editor spans and independently verified witness.
use davinci_harness::alloc::{self, CountingAllocator};
use std::{hint::black_box, time::Instant};
use vize_l0::{Allocator, Lang, Span};
use vize_l3::{
    op::{EffectId, OpId, RegionId},
    query::{Direction, EdgeKinds, OpGraph, ReachQuery, ReachStep},
};

mod profile {
    pub(super) mod checks;
    pub(super) mod trails;
}

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
        let mut old = Vec::new();
        let mut new = Vec::new();
        let mut measure = || {
            let sample = |joined| {
                let start = Instant::now();
                for _ in 0..64 {
                    black_box(read(joined));
                }
                start.elapsed().as_nanos() / 64
            };
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
            (old[old.len() / 2], new[new.len() / 2])
        };
        let (mut before_ns, mut after_ns) = measure();
        if after_ns > before_ns {
            println!(
                "graph {label} cold={cold} first paired median {before_ns}/{after_ns}; confirming with two more windows"
            );
            measure();
            (before_ns, after_ns) = measure();
        }
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

// Each measurement starts with an identical resident graph and no cached
// answers. The output digest is consumed inside the window so a broad scan
// need not keep all quadratic output data alive at once.
fn scan(
    source: &str,
    stream: bool,
    allocations: bool,
) -> ((usize, u64), Option<alloc::AllocMetrics>, u128) {
    use vize_incremental::{BlockKind, ResidentDatabase};
    use vize_resident_graph::{TemplateQueries, TemplateReach};
    let db = ResidentDatabase::default();
    let file = db.open("Batch.vue", source);
    let block = db
        .document_blocks(file)
        .unwrap()
        .iter()
        .find(|b| b.kind() == &BlockKind::Template)
        .unwrap();
    let batch = block.reach_batch(Lang::Js).unwrap();
    let run = || {
        let mut count = 0;
        let mut digest = 0u64;
        let mut consume = |answer: &TemplateReach| {
            for (op, span) in answer.spans() {
                count += 1;
                digest = digest.rotate_left(7)
                    ^ u64::from(op.index())
                    ^ u64::from(span.start)
                    ^ u64::from(span.end);
            }
        };
        for start in 0..256 {
            let query = ReachQuery {
                start: OpId::new(start),
                direction: Direction::Forward,
                kinds: EdgeKinds::ALL,
            };
            if stream {
                consume(batch.reachable(query).unwrap().relative());
            } else {
                consume(block.reachable(Lang::Js, query).unwrap().relative());
            }
        }
        (count, digest)
    };
    let start = Instant::now();
    let (result, cost) = if allocations {
        alloc::measure_returning(run)
    } else {
        (run(), None)
    };
    let elapsed = start.elapsed().as_nanos();
    (result, cost, elapsed)
}

fn batch_profile() {
    let source = vize_l0::cstr!("<template>{}</template>", "<p/>".repeat(256));
    let (old, old_alloc, _) = scan(&source, false, true);
    let (new, new_alloc, _) = scan(&source, true, true);
    assert_eq!(old, new);
    assert_eq!(new.0, 256 * 257 / 2);
    let (old_alloc, new_alloc) = (old_alloc.unwrap(), new_alloc.unwrap());
    assert!(new_alloc.peak_bytes_over_start < old_alloc.peak_bytes_over_start);
    let mut cached = Vec::new();
    let mut transient = Vec::new();
    for round in 0..15 {
        let first = round % 2 != 0;
        let a = scan(&source, first, false).2;
        let b = scan(&source, !first, false).2;
        if round >= 4 {
            let (old, new) = if first { (b, a) } else { (a, b) };
            cached.push(old);
            transient.push(new);
        }
    }
    cached.sort_unstable();
    transient.sort_unstable();
    println!(
        "resident 256 starts steps={} ns={}/{} calls={}/{} peak={}/{}",
        new.0,
        cached[5],
        transient[5],
        old_alloc.calls,
        new_alloc.calls,
        old_alloc.peak_bytes_over_start,
        new_alloc.peak_bytes_over_start
    );
}

fn main() {
    alloc::mark_installed();
    for siblings in [1, 4, 32, 128] {
        let source = "<p :title=\"title\">{{title}}</p>".repeat(siblings);
        let arena = Allocator::default();
        let native =
            vize_l1_to_l2::native::lower_component_native(&arena, &source, Lang::Js).unwrap();
        assert!(native.is_supported());
        let mut lowered = vize_l2_to_l3::lower(&arena, native.artifact.root());
        if siblings > 1 {
            profile::trails::profile(lowered.l3ir(), siblings);
        }
        profile::checks::profile(&mut lowered.program, siblings);
    }
    batch_profile();
    profile("<div/>", "one");
    profile(
        "<input :title=\"label\"><p>{{label}}</p><span>字</span>",
        "siblings",
    );
    let source = "<p :title=\"title\">{{title}}</p>".repeat(64);
    profile(&source, "64-siblings");
}
