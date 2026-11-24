use crate::{bounded_reference::scan, *};
use std::ops::ControlFlow;
use vize_l3::query::{BoundedTrailQuery, EdgePattern, TrailMatch, TrailVisit, TrailWitnessError};

fn bounded(
    start: OpId,
    lo: u32,
    hi: u32,
    direction: Direction,
    kinds: EdgeKinds,
) -> BoundedTrailQuery {
    BoundedTrailQuery {
        start,
        step: EdgePattern { direction, kinds },
        min_hops: lo,
        max_hops: hi,
    }
}

#[test]
fn every_ordered_prefix_matches_the_original_typed_relation() {
    use EdgeKind::{DataDependency as V, DomOrder as D, EffectOrder as E};
    let a = Allocator::default();
    let ir = build(
        &a,
        &[9, 0, u32::MAX],
        &[
            (0, 9, D),
            (0, 9, D),
            (9, 0, E),
            (9, 9, V),
            (9, u32::MAX, D),
            (u32::MAX, 0, E),
        ],
    );
    let graph = OpGraph::new(&ir).unwrap();
    let mut tested = 0;
    for seed in &ir.ops {
        for d in [Direction::Forward, Direction::Reverse] {
            for mask in 0..8 {
                let kinds = [D, E, V]
                    .into_iter()
                    .enumerate()
                    .filter(|(i, _)| mask & (1 << i) != 0)
                    .fold(EdgeKinds::NONE, |k, (_, e)| k.union(EdgeKinds::only(e)));
                for lo in 0..=4 {
                    for hi in 0..=5 {
                        let q = bounded(seed.id, lo, hi, d, kinds);
                        let mut expected = Vec::new();
                        scan(&ir, q, q.start, &mut Vec::new(), &mut expected);
                        let mut observed = Vec::new();
                        let result = graph
                            .visit_bounded_trails(q, usize::MAX, |r| {
                                assert_eq!(graph.check_bounded_trail(q, r), Ok(()));
                                observed.push((r.end, r.edges.to_vec()));
                                ControlFlow::<()>::Continue(())
                            })
                            .unwrap();
                        assert_eq!(result, TrailVisit::Complete);
                        assert_eq!(observed, expected);
                        let first = graph
                            .visit_bounded_trails(q, usize::MAX, |r| {
                                ControlFlow::Break((r.end, r.edges.to_vec()))
                            })
                            .unwrap();
                        assert_eq!(
                            first,
                            expected
                                .first()
                                .cloned()
                                .map_or(TrailVisit::Complete, TrailVisit::Break)
                        );
                        tested += 1;
                    }
                }
            }
        }
    }
    assert_eq!(tested, 1440);
}

#[test]
fn filtered_and_repeated_relationships_spend_budget_and_partial_rows_stay_partial() {
    use EdgeKind::{DomOrder as D, EffectOrder as E};
    let a = Allocator::default();
    let ir = build(&a, &[17, 29], &[(17, 17, E), (17, 17, D), (17, 29, D)]);
    let graph = OpGraph::new(&ir).unwrap();
    let seed = OpId::new(17);
    let q = bounded(seed, 0, 3, Direction::Forward, EdgeKinds::only(D));
    let run = |budget| {
        let mut rows = Vec::new();
        let end = graph
            .visit_bounded_trails(q, budget, |m| {
                rows.push(m.edges.to_vec());
                ControlFlow::<()>::Continue(())
            })
            .unwrap();
        (end, rows)
    };
    for (budget, expected) in [
        (0, vec![vec![]]),
        (1, vec![vec![]]),
        (2, vec![vec![], vec![1]]),
        (4, vec![vec![], vec![1]]),
        (5, vec![vec![], vec![1], vec![1, 2]]),
    ] {
        assert_eq!(run(budget), (TrailVisit::BudgetExhausted, expected));
    }
    assert_eq!(
        run(6),
        (
            TrailVisit::Complete,
            vec![vec![], vec![1], vec![1, 2], vec![2]]
        )
    );
    assert_eq!(
        graph
            .visit_bounded_trails(q, 0, |r| ControlFlow::Break(r.end))
            .unwrap(),
        TrailVisit::Break(seed)
    );
    let witness = |end, edges: &[u32]| graph.check_bounded_trail(q, TrailMatch { end, edges });
    assert_eq!(
        witness(seed, &[1, 1]),
        Err(TrailWitnessError::RepeatedEdge(1))
    );
    assert_eq!(witness(seed, &[0]), Err(TrailWitnessError::Step(0)));
    assert_eq!(
        witness(seed, &[u32::MAX]),
        Err(TrailWitnessError::MissingEdge(u32::MAX))
    );
    assert_eq!(
        witness(seed, &[2]),
        Err(TrailWitnessError::End {
            actual: OpId::new(29),
            claimed: seed
        })
    );
    assert_eq!(witness(seed, &[1; 4]), Err(TrailWitnessError::Length));
    assert_eq!(witness(seed, &[2, 1]), Err(TrailWitnessError::Step(1)));
    let unknown = bounded(OpId::new(33), 3, 0, Direction::Forward, EdgeKinds::NONE);
    assert_eq!(
        graph.visit_bounded_trails(unknown, 0, |_| ControlFlow::Break(())),
        Err(GraphError::UnknownOp(unknown.start))
    );
    assert_eq!(
        graph.check_bounded_trail(
            unknown,
            TrailMatch {
                end: unknown.start,
                edges: &[]
            }
        ),
        Err(TrailWitnessError::UnknownStart(unknown.start))
    );
}

#[test]
fn deep_chains_and_backtracking_cross_inline_storage_without_using_the_call_stack() {
    let a = Allocator::default();
    let last = 12_000u32;
    let mut edges: Vec<_> = (0..last).map(|n| (n, n + 1, EdgeKind::DomOrder)).collect();
    edges.push((last, 0, EdgeKind::DomOrder));
    edges.push((last, 0, EdgeKind::DomOrder));
    let ir = build(&a, &(0..=last).collect::<Vec<_>>(), &edges);
    let graph = OpGraph::new(&ir).unwrap().into_owned();
    let q = bounded(
        OpId::new(0),
        last + 1,
        u32::MAX,
        Direction::Forward,
        EdgeKinds::ALL,
    );
    let mut leaves = Vec::new();
    let status = graph
        .visit_bounded_trails(q, usize::MAX, |r| {
            assert_eq!(r.end, q.start);
            assert_eq!(graph.check_bounded_trail(q, r), Ok(()));
            leaves.push(*r.edges.last().unwrap());
            ControlFlow::<()>::Continue(())
        })
        .unwrap();
    assert_eq!(status, TrailVisit::Complete);
    assert_eq!(leaves, [last, last + 1]);
}
