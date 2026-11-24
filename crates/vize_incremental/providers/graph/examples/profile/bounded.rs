//! Read relationship identities for a compiler pass and current operation spans
//! for an editor pass, once separately and once on the same borrowed trail.
use davinci_harness::{alloc, paired::assert_paired_time};
use std::{hint::black_box, ops::ControlFlow};
use vize_l3::{
    L3Ir,
    query::{BoundedTrailQuery, Direction, EdgeKinds, EdgePattern, OpGraph, TrailVisit},
};

#[derive(Default, Debug, PartialEq, Eq)]
struct Products {
    trails: (usize, u64),
    sites: (usize, u64),
}

fn consume<const EDGES: bool, const SITES: bool>(
    graph: &OpGraph<'_>,
    q: BoundedTrailQuery,
    first: bool,
    out: &mut Products,
) {
    let outcome = graph
        .visit_bounded_trails(q, usize::MAX, |row| {
            if EDGES {
                out.trails.0 += 1;
                let mut value = u64::from(row.end.index());
                for ordinal in row.edges {
                    value = value.rotate_left(7) ^ u64::from(*ordinal);
                }
                out.trails.1 = out.trails.1.rotate_left(9) ^ value;
            }
            if SITES {
                out.sites.0 += 1;
                let span = graph.operation(row.end).unwrap().span;
                out.sites.1 = out.sites.1.rotate_left(11)
                    ^ (u64::from(span.start) << 32 | u64::from(span.end));
            }
            if first {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        })
        .unwrap();
    assert!(matches!(
        outcome,
        TrailVisit::Complete | TrailVisit::Break(())
    ));
}

pub(crate) fn profile(ir: &L3Ir<'_>, siblings: usize) {
    let graph = OpGraph::new(ir).unwrap();
    let requests: Vec<_> = ir
        .ops
        .iter()
        .flat_map(|op| {
            [Direction::Forward, Direction::Reverse].map(|direction| BoundedTrailQuery {
                start: op.id,
                step: EdgePattern {
                    direction,
                    kinds: EdgeKinds::ALL,
                },
                min_hops: 1,
                max_hops: 4,
            })
        })
        .collect();
    for cold in [false, true] {
        for first in [false, true] {
            let run = |together| {
                let new_index;
                let index = if cold {
                    new_index = OpGraph::new(black_box(ir)).unwrap();
                    &new_index
                } else {
                    black_box(&graph)
                };
                let mut products = Products::default();
                for &q in black_box(&requests) {
                    if together {
                        consume::<true, true>(index, q, first, &mut products);
                    } else {
                        consume::<true, false>(index, q, first, &mut products);
                        consume::<false, true>(index, q, first, &mut products);
                    }
                }
                products
            };
            let (old, a) = alloc::measure_returning(|| run(false));
            let (new, b) = alloc::measure_returning(|| run(true));
            assert_eq!(old, new);
            assert!(new.trails.0 > 0 && new.trails.0 == new.sites.0);
            let (a, b) = (a.unwrap(), b.unwrap());
            assert!(b.calls <= a.calls && b.peak_bytes_over_start <= a.peak_bytes_over_start);
            let name = format!("bounded trails siblings={siblings} cold={cold} first={first}");
            println!(
                "{name} requests={} rows={} calls={}/{} peak={}/{}",
                requests.len(),
                new.trails.0,
                a.calls,
                b.calls,
                a.peak_bytes_over_start,
                b.peak_bytes_over_start
            );
            assert_paired_time(&name, 8, |joined| {
                black_box(run(joined));
            });
        }
    }
}
