use crate::*;
use std::ops::ControlFlow;
use vize_l3::query::{EdgePattern, TrailError, TwoHopMatch, TwoHopQuery, check_two_hop_matches};

fn patterns() -> impl Iterator<Item = EdgePattern> {
    [Direction::Forward, Direction::Reverse]
        .into_iter()
        .flat_map(|direction| {
            (0..8).map(move |mask| {
                let kinds = [
                    EdgeKind::DomOrder,
                    EdgeKind::EffectOrder,
                    EdgeKind::DataDependency,
                ]
                .into_iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .fold(EdgeKinds::NONE, |kinds, (_, kind)| {
                    kinds.union(EdgeKinds::only(kind))
                });
                EdgePattern { direction, kinds }
            })
        })
}

fn matches(graph: &OpGraph<'_>, query: TwoHopQuery) -> Vec<TwoHopMatch> {
    let mut out = Vec::new();
    assert!(
        graph
            .visit_two_hop(query, |m| {
                out.push(m);
                ControlFlow::<()>::Continue(())
            })
            .unwrap()
            .is_continue()
    );
    out
}

#[test]
fn every_kind_mask_and_direction_preserves_parallel_relationship_multiplicity() {
    use EdgeKind::{DataDependency as Data, DomOrder as Dom, EffectOrder as Effect};
    let a = Allocator::default();
    let ir = build(
        &a,
        &[10, 0, 81, u32::MAX],
        &[
            (0, 10, Dom),
            (0, 10, Dom),
            (10, 0, Data),
            (10, 10, Effect),
            (10, 81, Dom),
        ],
    );
    let graph = OpGraph::new(&ir).unwrap();
    for start in &ir.ops {
        for first in patterns() {
            for second in patterns() {
                let query = TwoHopQuery {
                    start: start.id,
                    first,
                    second,
                };
                let rows = matches(&graph, query);
                assert_eq!(check_two_hop_matches(&ir, query, &rows), Ok(()));
                assert_eq!(graph.check_two_hop(query, &rows), Ok(()));
                let first = graph
                    .visit_two_hop(query, ControlFlow::Break)
                    .unwrap()
                    .break_value();
                assert_eq!(first.as_ref(), rows.first());
            }
        }
    }
    let dom = EdgePattern {
        direction: Direction::Forward,
        kinds: EdgeKinds::only(Dom),
    };
    let query = TwoHopQuery {
        start: OpId::new(0),
        first: dom,
        second: dom,
    };
    let rows = matches(&graph, query);
    assert_eq!(
        rows.iter()
            .map(|m| (m.first_edge, m.second_edge, m.middle.index(), m.end.index()))
            .collect::<Vec<_>>(),
        [(0, 4, 10, 81), (1, 4, 10, 81)]
    );
    let backwards = TwoHopQuery {
        second: EdgePattern {
            direction: Direction::Reverse,
            ..dom
        },
        ..query
    };
    assert_eq!(
        matches(&graph, backwards)
            .iter()
            .map(|m| (m.first_edge, m.second_edge, m.end.index()))
            .collect::<Vec<_>>(),
        [(0, 1, 0), (1, 0, 0)]
    );
    let owned = graph.clone().into_owned();
    let compare = |query, rows: &[TwoHopMatch]| {
        let reference = check_two_hop_matches(&ir, query, rows);
        assert_eq!(graph.check_two_hop(query, rows), reference);
        assert_eq!(owned.check_two_hop(query, rows), reference);
        reference
    };
    for altered in [
        vec![rows[0]],
        vec![rows[1], rows[0]],
        vec![rows[0], rows[0]],
        vec![
            TwoHopMatch {
                end: OpId::new(0),
                ..rows[0]
            },
            rows[1],
        ],
        vec![
            TwoHopMatch {
                middle: OpId::new(81),
                ..rows[0]
            },
            rows[1],
        ],
        vec![
            TwoHopMatch {
                second_edge: rows[0].first_edge,
                ..rows[0]
            },
            rows[1],
        ],
    ] {
        assert!(matches!(
            compare(query, &altered),
            Err(TrailError::Mismatch { .. })
        ));
    }
    let mut extra = rows.clone();
    extra.push(rows[0]);
    assert_eq!(compare(query, &extra), Err(TrailError::Extra { at: 2 }));
    let absent = TwoHopQuery {
        start: OpId::new(9),
        ..query
    };
    assert_eq!(
        graph.visit_two_hop(absent, ControlFlow::Break),
        Err(GraphError::UnknownOp(absent.start))
    );
    assert_eq!(
        compare(absent, &[]),
        Err(TrailError::Graph(GraphError::UnknownOp(absent.start)))
    );
    let dangling = build(&a, &[0, 1], &[(1, 8, Effect)]);
    assert_eq!(
        check_two_hop_matches(&dangling, absent, &[]),
        Err(TrailError::Graph(GraphError::UnknownOp(absent.start)))
    );
    assert!(matches!(
        check_two_hop_matches(&dangling, query, &[]),
        Err(TrailError::Graph(GraphError::MissingEndpoint { .. }))
    ));
}
