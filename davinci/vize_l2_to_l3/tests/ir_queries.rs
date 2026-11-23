#![cfg(feature = "graph-queries")]

use std::collections::BTreeSet;
use vize_l0::{Allocator, Span};
use vize_l3::{
    L3Ir,
    op::{EdgeKind, Op, OpId, OpKind, Phase, RegionId, StateEdge},
    query::{
        CertificateError, Direction, EdgeKinds, GraphError, OpGraph, ReachQuery, ReachStep,
        check_reachability, incident,
    },
};

fn build<'a>(arena: &'a Allocator, ops: &[u32], edges: &[(u32, u32, EdgeKind)]) -> L3Ir<'a> {
    let mut ir = L3Ir::new(arena, Phase::Built);
    for &id in ops {
        ir.push_op(Op::new(
            OpId::new(id),
            OpKind::InsertNode,
            RegionId::ROOT,
            Span::new(0, 0),
        ));
    }
    for &(from, to, kind) in edges {
        ir.push_edge(StateEdge::new(OpId::new(from), OpId::new(to), kind));
    }
    ir
}

fn query(start: u32) -> ReachQuery {
    ReachQuery {
        start: OpId::new(start),
        direction: Direction::Forward,
        kinds: EdgeKinds::ALL,
    }
}

fn fixed_point(ir: &L3Ir<'_>, query: ReachQuery) -> BTreeSet<OpId> {
    let mut set = BTreeSet::from([query.start]);
    loop {
        let size = set.len();
        for edge in &ir.edges {
            let (from, to) = match query.direction {
                Direction::Forward => (edge.from, edge.to),
                Direction::Reverse => (edge.to, edge.from),
            };
            if query.kinds.contains(edge.kind) && set.contains(&from) {
                set.insert(to);
            }
        }
        if size == set.len() {
            return set;
        }
    }
}

fn differential(ir: &L3Ir<'_>) {
    use EdgeKind::{DataDependency as Data, DomOrder as Dom, EffectOrder as Effect};
    let index = OpGraph::new(ir).unwrap();
    let retained = index.clone().into_owned();
    for op in &ir.ops {
        for direction in [Direction::Forward, Direction::Reverse] {
            for mask in 0..8 {
                let mut kinds = EdgeKinds::NONE;
                for (bit, kind) in [Dom, Effect, Data].into_iter().enumerate() {
                    if mask & (1 << bit) != 0 {
                        kinds = kinds.union(EdgeKinds::only(kind));
                    }
                }
                let query = ReachQuery {
                    start: op.id,
                    direction,
                    kinds,
                };
                let scan: Vec<_> = incident(ir, query)
                    .map(|(i, edge)| (i as u32, edge))
                    .collect();
                assert_eq!(index.incident(query).unwrap().collect::<Vec<_>>(), scan);
                let mut visited = Vec::new();
                let steps = index.reach_with(query, |op| visited.push(op.id)).unwrap();
                assert_eq!(visited, steps.iter().map(|s| s.op).collect::<Vec<_>>());
                assert_eq!(
                    visited.len(),
                    visited.iter().copied().collect::<BTreeSet<_>>().len()
                );
                assert_eq!(
                    visited.into_iter().collect::<BTreeSet<_>>(),
                    fixed_point(ir, query)
                );
                assert_eq!(retained.reach_with(query, |_| {}).unwrap(), steps);
                assert_eq!(retained.check(query, &steps), Ok(()));
                assert_eq!(check_reachability(ir, query, &steps), Ok(()));
            }
        }
    }
}

#[test]
fn cycles_parallel_edges_and_sparse_ids_preserve_every_selected_original_edge() {
    use EdgeKind::{DataDependency as Data, DomOrder as Dom, EffectOrder as Effect};
    let arena = Allocator::default();
    let ir = build(
        &arena,
        &[81, 0, 2, 17, u32::MAX],
        &[
            (0, 2, Dom),
            (2, 2, Dom),
            (0, 2, Effect),
            (17, 2, Data),
            (2, 0, Data),
            (0, 17, Effect),
        ],
    );
    differential(&ir);
    let graph = OpGraph::new(&ir).unwrap();
    let steps = graph.reach_with(query(0), |_| {}).unwrap();
    assert_eq!(steps, [step(0, None), step(2, Some(0)), step(17, Some(5))]);
    assert_eq!(
        graph.check(
            query(0),
            &[step(0, None), step(17, Some(5)), step(2, Some(3))]
        ),
        Ok(())
    );
}

fn step(op: u32, via: Option<u32>) -> ReachStep {
    ReachStep {
        op: OpId::new(op),
        via,
    }
}

#[test]
fn checker_rejects_corrupted_witnesses_even_when_the_claimed_set_looks_plausible() {
    use EdgeKind::{DataDependency as Data, DomOrder as Dom};
    let arena = Allocator::default();
    let ir = build(
        &arena,
        &[0, 1, 2, 3, 4],
        &[
            (0, 1, Dom),
            (1, 2, Dom),
            (3, 4, Dom),
            (4, 3, Dom),
            (0, 2, Data),
        ],
    );
    let q = ReachQuery {
        kinds: EdgeKinds::only(Dom),
        ..query(0)
    };
    let root = step(0, None);
    let good = [root, step(1, Some(0)), step(2, Some(1))];
    let graph = OpGraph::new(&ir).unwrap();
    let owned = graph.clone().into_owned();
    let check = |rows: &[ReachStep]| {
        let reference = check_reachability(&ir, q, rows);
        assert_eq!(graph.check(q, rows), reference);
        assert_eq!(owned.check(q, rows), reference);
        reference
    };
    assert_eq!(check(&good), Ok(()));
    assert_eq!(check(&[]), Err(CertificateError::Root));
    assert_eq!(check(&[step(0, Some(0))]), Err(CertificateError::Root));
    assert_eq!(
        check(&[root, step(0, None)]),
        Err(CertificateError::Duplicate(OpId::new(0)))
    );
    assert_eq!(check(&[root]), Err(CertificateError::OpenEdge(0)));
    for bad in [
        vec![root, step(2, Some(1)), step(1, Some(0))],
        vec![root, step(1, Some(0)), step(2, Some(4))],
        vec![root, step(1, None)],
        vec![root, step(1, Some(90))],
        vec![
            root,
            step(1, Some(0)),
            step(2, Some(1)),
            step(3, Some(3)),
            step(4, Some(2)),
        ],
    ] {
        assert!(matches!(check(&bad), Err(CertificateError::Witness { .. })));
    }
    assert!(
        check_reachability(
            &ir,
            ReachQuery {
                direction: Direction::Reverse,
                ..q
            },
            &good
        )
        .is_err()
    );
    assert!(matches!(
        OpGraph::new(&ir).unwrap().reach_with(query(99), |_| {}),
        Err(GraphError::UnknownOp(_))
    ));
    let invalid = build(&arena, &[0, 1, 2], &[(2, 99, Data)]);
    assert!(matches!(
        OpGraph::new(&invalid),
        Err(GraphError::MissingEndpoint { .. })
    ));
    assert!(matches!(
        check_reachability(&invalid, q, &[root]),
        Err(CertificateError::Graph(GraphError::MissingEndpoint { .. }))
    ));
    let duplicate = build(&arena, &[21, 0, 21], &[]);
    assert!(matches!(
        OpGraph::new(&duplicate),
        Err(GraphError::DuplicateOp(_))
    ));
}

#[test]
fn reference_templates_match_at_every_start_in_both_directions_for_every_edge_mask() {
    let arena = Allocator::default();
    for source in [
        include_str!(
            "../../../tests/fixtures/l3-reference/rust-lowered-control-slots.template.txt"
        ),
        r#"<main class="shell"><button :disabled="locked" @click="save">{{ label }}</button></main>"#,
    ] {
        let (tree, errors) = vize_l1::markup::parse(&arena, source.trim_end());
        assert!(errors.is_empty());
        let l2 = vize_l1_to_l2::lower(&arena, &tree, &errors);
        let lowered = vize_l2_to_l3::lower(&arena, &l2.artifact.root);
        assert!(lowered.l3ir().ops.len() > 2);
        differential(lowered.l3ir());
    }
}

#[test]
fn original_l1_round_trips_recovered_source_and_only_admitted_l2_supplies_the_graph() {
    use vize_l0::config::{VueDialect, VueVersion};
    use vize_l1::{container::vue::DescriptorOptions, markup::SurfaceParseOptions};
    let arena = Allocator::default();
    let options = DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    };
    let clean = "\r\n<template><div :title=\"value\">{{value}}</div></template>\r\n<script setup lang=ts>/* 日本語 */ const value = 1;</script>";
    for (text, admits) in [
        (clean, true),
        ("\u{feff}<!--字-->\r\n<template><div", false),
        (
            "<script setup lang=ts>const value = 1;</script><template>{{value as number}}</template>",
            false,
        ),
    ] {
        let observed = vize_l1_to_l2::native_file::lower_sfc_native(&arena, text, options);
        let l1 = observed.l1ir();
        let mut cursor = 0;
        let rebuilt: String = l1
            .container()
            .segments()
            .map(|seg| {
                let span = seg.span();
                assert_eq!(span.start, cursor);
                cursor = span.end;
                l1.source()
                    .get(span.start as usize..span.end as usize)
                    .unwrap()
            })
            .collect();
        assert_eq!(rebuilt.as_bytes(), text.as_bytes());
        assert_eq!(cursor as usize, text.len());
        assert_eq!(observed.admitted().is_some(), admits);
        if let Some(native) = observed.admitted() {
            let file = native.l2ir();
            assert!(core::ptr::eq(file.source(), text));
            assert!(core::ptr::eq(file.root_region(), file.artifact().root()));
            let ir = vize_l2_to_l3::lower_ir(&arena, file);
            assert!(!ir.l3ir().ops.is_empty());
            differential(ir.l3ir());
        } else {
            assert!(!observed.issues().is_empty());
        }
    }
}

mod ir_queries {
    mod trails;
}
