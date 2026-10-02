//! Native laws over the real canonical owner, with no legacy facts input.

use vize_l0::{Allocator, Box, Vec, id::NodeId};
use vize_l2::{
    op::{CommentOp, DynamicName, ForBinding, ForOp, IfBranch, IfOp, Op, SlotOp},
    walk::NodeRef,
};
use vize_l2_to_l3::build_decisions;
use vize_l3::{
    decision::{ControlKind, StaticLevel, policy::TargetPolicy},
    placement::Placement,
};

mod decision_support;
use decision_support::{
    SPAN, bind, cloak, component, element, event, every_binding, expression, interpolation, region,
    seal, seal_scoped, text,
};

fn id(index: u32) -> NodeId {
    NodeId::from_index(index).expect("fixture id fits")
}

#[test]
fn every_attached_family_is_accounted_for_in_authored_order() {
    let allocator = &Allocator::default();
    let artifact = seal_scoped(
        region(
            &allocator,
            [element(
                &allocator,
                every_binding(&allocator),
                [text(&allocator)],
            )],
        ),
        [id(4), id(8)],
    );
    let dom_analysis = build_decisions(&artifact, TargetPolicy::Dom).expect("sealed owner");
    let dom = dom_analysis.tables();
    let ssr_analysis = build_decisions(&artifact, TargetPolicy::Ssr).expect("sealed owner");
    let ssr = ssr_analysis.tables();
    let vapor_analysis = build_decisions(&artifact, TargetPolicy::Vapor).expect("sealed owner");
    let vapor = vapor_analysis.tables();
    assert_eq!(artifact.node_count(), 16);
    assert_eq!(
        dom.nodes.get(id(0)).unwrap().dynamic_bindings,
        (1..14).map(id).collect::<std::vec::Vec<_>>()
    );
    assert_eq!(
        ssr.nodes.get(id(0)).unwrap().dynamic_bindings,
        (1..14)
            .filter(|index| *index != 2)
            .map(id)
            .collect::<std::vec::Vec<_>>()
    );
    assert_eq!(
        *vapor,
        vize_l3::decision::DecisionTables {
            policy: TargetPolicy::Vapor,
            ..dom.clone()
        }
    );
    let mut region_ops = 0;
    let mut attached_ops = 0;
    artifact
        .visit_nodes(&mut |node, borrowed| {
            match borrowed {
                NodeRef::Op(_) => region_ops += 1,
                NodeRef::Binding(_) => attached_ops += 1,
            }
            for table in [&dom, &ssr, &vapor] {
                let row = table
                    .nodes
                    .get(node)
                    .expect("every actual artifact id has one row");
                assert_eq!(row.static_level, dom.nodes.get(node).unwrap().static_level);
                assert_eq!(row.placement, Placement::Inline);
                assert_eq!(row.control, None);
            }
        })
        .expect("sealed owner");
    assert_eq!((region_ops, attached_ops), (2, 14));
    for table in [&dom, &ssr, &vapor] {
        assert_eq!(table.nodes.len(), 16);
        assert!(table.controls.is_empty());
    }
    assert_eq!(
        dom.nodes.get(id(14)).unwrap().static_level,
        StaticLevel::Static
    );
    assert_eq!(
        ssr.nodes.get(id(2)).unwrap().output_level,
        StaticLevel::Static
    );
}

#[test]
fn output_filtering_propagates_to_ancestors_without_changing_neutral_meaning() {
    let allocator = &Allocator::default();
    let artifact = seal(region(
        &allocator,
        [
            element(
                &allocator,
                [],
                [element(&allocator, [event(&allocator)], [text(&allocator)])],
            ),
            component(&allocator, [event(&allocator)]),
        ],
    ));
    let dom_analysis = build_decisions(&artifact, TargetPolicy::Dom).unwrap();
    let dom = dom_analysis.tables();
    let ssr_analysis = build_decisions(&artifact, TargetPolicy::Ssr).unwrap();
    let ssr = ssr_analysis.tables();
    for index in 0..3 {
        assert_eq!(
            dom.nodes.get(id(index)).unwrap().static_level,
            StaticLevel::Dynamic
        );
        assert_eq!(
            ssr.nodes.get(id(index)).unwrap().static_level,
            StaticLevel::Dynamic
        );
        assert_eq!(
            dom.nodes.get(id(index)).unwrap().output_level,
            StaticLevel::Dynamic
        );
        assert_eq!(
            ssr.nodes.get(id(index)).unwrap().output_level,
            StaticLevel::Static
        );
    }
    assert_eq!(dom.nodes.get(id(1)).unwrap().dynamic_bindings, [id(2)]);
    assert!(ssr.nodes.get(id(1)).unwrap().dynamic_bindings.is_empty());
    assert_eq!(
        ssr.nodes.get(id(4)).unwrap().output_level,
        StaticLevel::Dynamic
    );
    assert_eq!(ssr.nodes.get(id(4)).unwrap().dynamic_bindings, [id(5)]);
    assert_eq!(
        ssr.nodes.get(id(5)).unwrap().output_level,
        StaticLevel::Dynamic
    );
}

#[test]
fn only_direct_interpolation_has_dynamic_text_classification() {
    let allocator = &Allocator::default();
    let artifact = seal(region(
        &allocator,
        [
            element(
                &allocator,
                [],
                [element(&allocator, [], [interpolation(&allocator)])],
            ),
            element(&allocator, [cloak(&allocator)], [text(&allocator)]),
            element(
                &allocator,
                [],
                [Op::Comment(Box::new_in(
                    CommentOp {
                        content: "authored",
                        span: SPAN,
                    },
                    &allocator,
                ))],
            ),
        ],
    ));
    for policy in [TargetPolicy::Dom, TargetPolicy::Ssr, TargetPolicy::Vapor] {
        let analysis = build_decisions(&artifact, policy).unwrap();
        let table = analysis.tables();
        for (index, expected) in [
            (0, StaticLevel::Dynamic),
            (1, StaticLevel::DynamicText),
            (2, StaticLevel::DynamicText),
            (3, StaticLevel::Static),
            (4, StaticLevel::Static),
            (5, StaticLevel::Static),
            (6, StaticLevel::Dynamic),
            (7, StaticLevel::Dynamic),
        ] {
            let row = table.nodes.get(id(index)).unwrap();
            assert_eq!(row.static_level, expected);
            assert_eq!(row.output_level, expected);
            assert!(row.dynamic_bindings.is_empty());
        }
    }
}

#[test]
fn nested_if_for_and_slot_fallback_preserve_control_containment() {
    let allocator = &Allocator::default();
    let expr = expression(&allocator);
    let slot = Op::Slot(Box::new_in(
        SlotOp {
            name: DynamicName::Static("default"),
            attributes: Vec::new_in(&allocator),
            bindings: Vec::from_iter_in([bind(&allocator), event(&allocator)], &allocator),
            fallback: region(
                &allocator,
                [element(&allocator, [event(&allocator)], [text(&allocator)])],
            ),
            span: SPAN,
        },
        &allocator,
    ));
    let for_op = Op::For(Box::new_in(
        ForOp {
            binding: ForBinding {
                source: expr,
                value: expr,
                key: None,
                index: None,
            },
            region: region(&allocator, [slot]),
            span: SPAN,
        },
        &allocator,
    ));
    let if_op = Op::If(Box::new_in(
        IfOp {
            branches: Vec::from_iter_in(
                [
                    IfBranch {
                        condition: Some(expr),
                        region: region(&allocator, [for_op]),
                        span: SPAN,
                    },
                    IfBranch {
                        condition: None,
                        region: region(
                            &allocator,
                            [Op::Comment(Box::new_in(
                                CommentOp {
                                    content: "else",
                                    span: SPAN,
                                },
                                &allocator,
                            ))],
                        ),
                        span: SPAN,
                    },
                ],
                &allocator,
            ),
            span: SPAN,
        },
        &allocator,
    ));
    let artifact = seal_scoped(region(&allocator, [if_op, text(&allocator)]), [id(1)]);
    assert_eq!(artifact.node_count(), 10);
    for policy in [TargetPolicy::Dom, TargetPolicy::Ssr, TargetPolicy::Vapor] {
        let analysis = build_decisions(&artifact, policy).unwrap();
        let table = analysis.tables();
        assert_eq!(table.nodes.len(), 10);
        assert_eq!(table.controls.len(), 3);
        for (index, kind, parent) in [
            (0, ControlKind::Conditional, None),
            (1, ControlKind::Loop, Some(id(0))),
            (2, ControlKind::Slot, Some(id(1))),
        ] {
            let control = table.controls.get(id(index)).unwrap();
            assert_eq!((control.kind, control.parent), (kind, parent));
        }
        for (index, control) in [
            (0, None),
            (1, Some(id(0))),
            (2, Some(id(1))),
            (3, Some(id(2))),
            (4, Some(id(2))),
            (5, Some(id(2))),
            (6, Some(id(2))),
            (7, Some(id(2))),
            (8, Some(id(0))),
            (9, None),
        ] {
            assert_eq!(table.nodes.get(id(index)).unwrap().control, control);
        }
        assert_eq!(
            table.nodes.get(id(2)).unwrap().dynamic_bindings,
            [id(3), id(4)]
        );
        let native_events = &table.nodes.get(id(5)).unwrap().dynamic_bindings;
        let expected_native_events: &[NodeId] = if policy == TargetPolicy::Ssr {
            &[]
        } else {
            &[id(6)]
        };
        assert_eq!(native_events.as_slice(), expected_native_events);
    }
}

#[test]
fn empty_owner_is_a_complete_empty_analysis_for_each_policy() {
    let allocator = &Allocator::default();
    let artifact = seal(region(&allocator, []));
    for policy in [TargetPolicy::Dom, TargetPolicy::Ssr, TargetPolicy::Vapor] {
        let analysis = build_decisions(&artifact, policy).unwrap();
        let table = analysis.tables();
        assert_eq!(table.policy, policy);
        assert!(table.nodes.is_empty());
        assert!(table.controls.is_empty());
    }
}

#[test]
fn analysis_retains_the_actual_owner_despite_equal_local_node_indices() {
    let allocator = &Allocator::default();
    let static_owner = seal(region(&allocator, [text(&allocator)]));
    let dynamic_owner = seal(region(&allocator, [interpolation(&allocator)]));
    assert_eq!(static_owner.node_count(), dynamic_owner.node_count());
    let static_analysis = build_decisions(&static_owner, TargetPolicy::Ssr).unwrap();
    let dynamic_analysis =
        vize_l3::decision::build_decisions(&dynamic_owner, TargetPolicy::Dom).unwrap();
    assert!(std::ptr::eq(static_analysis.artifact(), &static_owner));
    assert!(std::ptr::eq(dynamic_analysis.artifact(), &dynamic_owner));
    assert!(!std::ptr::eq(
        static_analysis.artifact(),
        dynamic_analysis.artifact()
    ));
    assert_eq!(static_analysis.policy(), TargetPolicy::Ssr);
    assert_eq!(dynamic_analysis.policy(), TargetPolicy::Dom);
    assert_eq!(
        static_analysis
            .tables()
            .nodes
            .get(id(0))
            .unwrap()
            .output_level,
        StaticLevel::Static
    );
    assert_eq!(
        dynamic_analysis
            .tables()
            .nodes
            .get(id(0))
            .unwrap()
            .output_level,
        StaticLevel::DynamicText
    );
    let mut detached_scratch = static_analysis.tables().clone();
    detached_scratch.nodes.clear();
    detached_scratch.policy = TargetPolicy::Vapor;
    assert_eq!(static_analysis.tables().nodes.len(), 1);
    assert_eq!(static_analysis.policy(), TargetPolicy::Ssr);
}
