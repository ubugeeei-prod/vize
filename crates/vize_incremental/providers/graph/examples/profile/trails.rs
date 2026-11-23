//! Compare the same typed join on the same resident index with and without
//! materializing the first projection. Neither measurement stores output rows.
use davinci_harness::alloc;
use std::{hint::black_box, ops::ControlFlow, time::Instant};
use vize_l3::{
    L3Ir,
    op::{OpId, StateEdge},
    query::{Direction, EdgeKinds, EdgePattern, OpGraph, ReachQuery, TwoHopMatch, TwoHopQuery},
};

fn dest(direction: Direction, edge: &StateEdge) -> OpId {
    match direction {
        Direction::Forward => edge.to,
        Direction::Reverse => edge.from,
    }
}

fn materialized(graph: &OpGraph<'_>, query: TwoHopQuery, mut visit: impl FnMut(TwoHopMatch)) {
    let request = |start, step: EdgePattern| ReachQuery {
        start,
        direction: step.direction,
        kinds: step.kinds,
    };
    let middle: Vec<_> = graph
        .incident(request(query.start, query.first))
        .unwrap()
        .map(|(ordinal, edge)| (ordinal, dest(query.first.direction, edge)))
        .collect();
    for (first_edge, middle) in middle {
        for (second_edge, edge) in graph.incident(request(middle, query.second)).unwrap() {
            if first_edge != second_edge {
                visit(TwoHopMatch {
                    first_edge,
                    second_edge,
                    middle,
                    end: dest(query.second.direction, edge),
                });
            }
        }
    }
}

fn visit(graph: &OpGraph<'_>, query: TwoHopQuery, stream: bool, mut sink: impl FnMut(TwoHopMatch)) {
    if stream {
        let flow = graph
            .visit_two_hop(query, |row| {
                sink(row);
                ControlFlow::<()>::Continue(())
            })
            .unwrap();
        assert_eq!(flow, ControlFlow::Continue(()));
    } else {
        materialized(graph, query, sink);
    }
}

pub(crate) fn profile(ir: &L3Ir<'_>, siblings: usize) {
    let graph = OpGraph::new(ir).unwrap();
    let first = EdgePattern {
        direction: Direction::Forward,
        kinds: EdgeKinds::ALL,
    };
    let queries = ir
        .ops
        .iter()
        .flat_map(|op| {
            [Direction::Forward, Direction::Reverse].map(|direction| TwoHopQuery {
                start: op.id,
                first,
                second: EdgePattern { direction, ..first },
            })
        })
        .collect::<Vec<_>>();
    for &query in &queries {
        let rows = |stream| {
            let mut rows = Vec::new();
            visit(&graph, query, stream, |m| rows.push(m));
            rows
        };
        assert_eq!(rows(false), rows(true));
    }
    let run = |stream| {
        let mut rows = 0usize;
        let mut digest = 0u64;
        for &query in black_box(&queries) {
            visit(black_box(&graph), query, stream, |m| {
                rows += 1;
                let edges = (u64::from(m.first_edge) << 32) | u64::from(m.second_edge);
                let ops = (u64::from(m.middle.index()) << 32) | u64::from(m.end.index());
                digest = digest.rotate_left(11) ^ edges ^ ops.rotate_left(7);
            });
        }
        (rows, digest)
    };
    let (expected, old) = alloc::measure_returning(|| run(false));
    let (actual, new) = alloc::measure_returning(|| run(true));
    assert_eq!(expected, actual);
    assert!(actual.0 > 0);
    let (old, new) = (old.unwrap(), new.unwrap());
    assert!(old.calls > 0);
    assert_eq!((new.calls, new.peak_bytes_over_start), (0, 0));
    let time = |stream| {
        let start = Instant::now();
        for _ in 0..64 {
            black_box(run(stream));
        }
        start.elapsed().as_nanos() / 64
    };
    let mut a = Vec::new();
    let mut b = Vec::new();
    for round in 0..35 {
        let stream_first = round % 2 != 0;
        let x = time(stream_first);
        let y = time(!stream_first);
        if round >= 4 {
            let (old, new) = if stream_first { (y, x) } else { (x, y) };
            a.push(old);
            b.push(new);
        }
    }
    a.sort_unstable();
    b.sort_unstable();
    let (materialized, streamed) = (a[a.len() / 2], b[b.len() / 2]);
    println!(
        "trails siblings={siblings} queries={} rows={} ns={materialized}/{streamed} calls={}/{} peak={}/{}",
        queries.len(),
        actual.0,
        old.calls,
        new.calls,
        old.peak_bytes_over_start,
        new.peak_bytes_over_start
    );
    assert!(streamed <= materialized);
}
