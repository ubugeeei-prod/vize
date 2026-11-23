//! Compare complete claims against the original relation with and without
//! retaining the source graph's validated operation positions across requests.
use davinci_harness::alloc;
use std::{hint::black_box, ops::ControlFlow, time::Instant};
use vize_l3::{
    L3Ir,
    op::OpId,
    query::{
        Direction, EdgeKinds, EdgePattern, OpGraph, ReachQuery, TwoHopQuery, check_reachability,
        check_two_hop_matches,
    },
};

fn measure(label: &str, mut run: impl FnMut(bool)) {
    let mut cost = |retained| {
        let ((), cost) = alloc::measure_returning(|| run(retained));
        cost.unwrap()
    };
    let (old, new) = (cost(false), cost(true));
    assert!(new.calls <= old.calls && new.peak_bytes_over_start <= old.peak_bytes_over_start);
    let mut time = |retained| {
        let start = Instant::now();
        for _ in 0..64 {
            run(retained);
        }
        start.elapsed().as_nanos()
    };
    let mut baseline = Vec::new();
    let mut candidate = Vec::new();
    for round in 0..35 {
        let first = round % 2 == 1;
        let a = time(first);
        let b = time(!first);
        if round >= 4 {
            let (before, after) = if first { (b, a) } else { (a, b) };
            baseline.push(before);
            candidate.push(after);
        }
    }
    baseline.sort_unstable();
    candidate.sort_unstable();
    println!(
        "checked {label} ns={}/{} calls={}/{} peak={}/{}",
        baseline[15] / 64,
        candidate[15] / 64,
        old.calls,
        new.calls,
        old.peak_bytes_over_start,
        new.peak_bytes_over_start
    );
    assert!(candidate[15] <= baseline[15]);
}

fn profile_ir(ir: &L3Ir<'_>, name: &str) {
    let graph = OpGraph::new(ir).unwrap().into_owned();
    let mut reach = Vec::new();
    let mut trails = Vec::new();
    for at in [0, ir.ops.len() / 2, ir.ops.len() - 1] {
        for direction in [Direction::Forward, Direction::Reverse] {
            let start = ir.ops[at].id;
            let kinds = EdgeKinds::ALL;
            let seed = ReachQuery {
                start,
                direction,
                kinds,
            };
            let steps = graph.reach_with(seed, |_| {}).unwrap();
            reach.push((seed, steps));
            let hop = EdgePattern { direction, kinds };
            let query = TwoHopQuery {
                start,
                first: hop,
                second: hop,
            };
            let mut rows = Vec::new();
            let completed = graph
                .visit_two_hop(query, |row| {
                    rows.push(row);
                    ControlFlow::<()>::Continue(())
                })
                .unwrap();
            assert!(completed.is_continue());
            trails.push((query, rows));
        }
    }
    let singletons: Vec<_> = reach
        .iter()
        .map(|(query, steps)| {
            let root = &steps[..1];
            (*query, root, check_reachability(ir, *query, root))
        })
        .collect();
    measure(&format!("{name} singleton"), |singleton| {
        for (query, root, expected) in black_box(&singletons) {
            let result = if singleton {
                black_box(&graph).check_singleton(*query)
            } else {
                black_box(&graph).check(*query, root)
            };
            assert_eq!(&result, expected);
        }
    });
    measure(&format!("{name} reach"), |retained| {
        for (query, claim) in black_box(&reach) {
            let result = if retained {
                black_box(&graph).check(*query, claim)
            } else {
                check_reachability(black_box(ir), *query, claim)
            };
            black_box(result).unwrap();
        }
    });
    measure(&format!("{name} trails"), |retained| {
        for (query, claim) in black_box(&trails) {
            let result = if retained {
                black_box(&graph).check_two_hop(*query, claim)
            } else {
                check_two_hop_matches(black_box(ir), *query, claim)
            };
            black_box(result).unwrap();
        }
    });
}

pub(crate) fn profile(ir: &mut L3Ir<'_>, siblings: usize) {
    profile_ir(ir, &format!("{siblings} dense"));
    // Preserve the same genuine operations and relations while exercising the
    // public query contract for sparse, descending operation identifiers.
    let relabel = |id: OpId| OpId::new(u32::MAX - 2 * id.index());
    for op in &mut ir.ops {
        op.id = relabel(op.id);
    }
    for edge in &mut ir.edges {
        edge.from = relabel(edge.from);
        edge.to = relabel(edge.to);
    }
    profile_ir(ir, &format!("{siblings} sparse"));
}
