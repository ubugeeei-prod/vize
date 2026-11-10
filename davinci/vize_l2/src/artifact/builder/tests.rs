extern crate std;

use super::Builder;
use crate::artifact::{Artifact, ArtifactError, LoweringFeatures, OpFamily};
use crate::expr::{ExprRef, JsExpr};
use crate::op::{Attribute, Namespace, Op};
use crate::provenance::ProvenanceRecord;
use crate::walk::{NodeEvent, NodeRef};
use vize_l0::{Allocator, Span, String, Vec, id::NodeId};

#[test]
fn construction_mints_exact_preorder_and_seals_without_a_second_walk() {
    let a = Allocator::default();
    let source = "<div>x<Child>{{value}}</Child></div>";
    let mut builder = Builder::new(&a, source).unwrap();
    let js = JsExpr::parse_in(&a, "value", Span::new(15, 20)).unwrap();
    builder
        .element(
            "div",
            Namespace::Html,
            Vec::new_in(&&a),
            Span::new(0, 36),
            |region, owner| {
                assert_eq!(owner, NodeId::FIRST);
                assert_eq!(region.text("x", Span::new(5, 6)).unwrap().index(), 1);
                region
                    .component(
                        "Child",
                        Vec::new_in(&&a),
                        Span::new(6, 30),
                        |region, child| {
                            assert_eq!(child.index(), 2);
                            assert_eq!(
                                region
                                    .interpolation(ExprRef::Js(js), Span::new(13, 22))
                                    .unwrap()
                                    .index(),
                                3
                            );
                        },
                    )
                    .unwrap();
            },
        )
        .unwrap();
    let artifact = builder.finish().unwrap();
    assert_eq!(artifact.node_count(), 4);
    let mut observed = alloc::vec::Vec::new();
    artifact
        .visit_events(&mut |event| {
            if let NodeEvent::Enter {
                id,
                node: NodeRef::Op(op),
                ..
            } = event
            {
                observed.push((id.index(), op.mnemonic()));
            }
        })
        .unwrap();
    assert_eq!(
        observed,
        [
            (0, "ui.element"),
            (1, "ui.text"),
            (2, "ui.component"),
            (3, "ui.interpolation")
        ]
    );
    // The independent arbitrary-parts checker agrees; production finish never
    // calls it and did not visit any previously constructed child again.
    assert_eq!(
        Artifact::try_new(artifact.into_parts())
            .unwrap()
            .node_count(),
        4
    );
}

#[test]
fn rejected_nodes_never_consume_an_id_or_replace_supported_fragments() {
    let a = Allocator::default();
    let mut builder = Builder::new(&a, "é").unwrap();
    assert!(matches!(
        builder.text("", Span::new(1, 2)),
        Err(ArtifactError::InvalidSpan { .. })
    ));
    assert_eq!(builder.text("é", Span::new(0, 2)).unwrap(), NodeId::FIRST);
    assert_eq!(builder.finish().unwrap().node_count(), 1);
}

#[test]
fn owner_spans_and_static_attribute_spans_are_checked_at_construction() {
    let a = Allocator::default();
    let mut builder = Builder::new(&a, "<x/>----").unwrap();
    let mut attrs = Vec::new_in(&&a);
    attrs.push(Attribute {
        name: "bad",
        value: None,
        span: Span::new(5, 7),
    });
    assert!(matches!(
        builder.element("x", Namespace::Html, attrs, Span::new(0, 4), |_, _| {}),
        Err(ArtifactError::OutsideOwner { .. })
    ));
    builder
        .element(
            "x",
            Namespace::Html,
            Vec::new_in(&&a),
            Span::new(0, 4),
            |region, _| {
                assert!(matches!(
                    region.text("--", Span::new(5, 7)),
                    Err(ArtifactError::OutsideOwner { .. })
                ));
            },
        )
        .unwrap();
    assert_eq!(builder.finish().unwrap().node_count(), 1);
}

#[test]
fn failed_decisions_remain_ordered_with_the_actual_partial_tree() {
    let a = Allocator::default();
    let mut builder = Builder::new(&a, "x?").unwrap();
    let id = builder.text("x", Span::new(0, 1)).unwrap();
    for (node, rule, span) in [
        (Some(id), "native.text", Span::new(0, 1)),
        (None, "native.unsupported", Span::new(1, 2)),
    ] {
        builder
            .record(ProvenanceRecord {
                rule: String::from(rule),
                node,
                before: String::from("source"),
                after: String::default(),
                span,
            })
            .unwrap();
    }
    let artifact = builder.finish().unwrap();
    assert_eq!(artifact.provenance().len(), 2);
    assert_eq!(artifact.provenance().get(1).unwrap().node, None);
    assert!(matches!(artifact.root().ops.first(), Some(Op::Text(_))));
}

#[test]
fn caught_callback_unwind_cannot_publish_a_false_seal_or_lose_partial_nodes() {
    let a = Allocator::default();
    let mut builder = Builder::new(&a, "<x>y</x>").unwrap();
    let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = builder.element(
            "x",
            Namespace::Html,
            Vec::new_in(&&a),
            Span::new(0, 8),
            |region, _| {
                region.text("y", Span::new(3, 4)).unwrap();
                panic!("producer callback");
            },
        );
    }));
    assert!(failed.is_err());
    let rejected = builder.finish().unwrap_err();
    assert_eq!(
        rejected.error,
        ArtifactError::UnfinishedOwner {
            node: NodeId::FIRST
        }
    );
    let restored = Artifact::try_new(*rejected.parts).unwrap();
    assert_eq!(restored.node_count(), 2);
    let Some(Op::Element(element)) = restored.root().ops.first() else {
        panic!("retained owner");
    };
    assert!(matches!(element.children.ops.first(), Some(Op::Text(_))));
}

#[test]
fn caught_nested_unwind_cannot_report_an_outer_owner_as_complete() {
    let a = Allocator::default();
    let source = "<x><C>z</C></x>";
    let mut builder = Builder::new(&a, source).unwrap();
    let outcome = builder.element(
        "x",
        Namespace::Html,
        Vec::new_in(&&a),
        Span::new(0, 15),
        |outer, _| {
            let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _ =
                    outer.component("C", Vec::new_in(&&a), Span::new(3, 11), |region, owner| {
                        assert_eq!(owner.index(), 1);
                        assert_eq!(region.text("z", Span::new(6, 7)).unwrap().index(), 2);
                        region
                            .record(ProvenanceRecord {
                                rule: String::from("native.pending.component"),
                                node: Some(owner),
                                before: String::from("<C>z</C>"),
                                after: String::default(),
                                span: Span::new(3, 11),
                            })
                            .unwrap();
                        panic!("nested producer callback");
                    });
            }));
            assert!(failed.is_err());
        },
    );
    let pending = NodeId::from_index(1).unwrap();
    assert_eq!(
        outcome,
        Err(ArtifactError::UnfinishedOwner { node: pending })
    );
    let rejected = builder.finish().unwrap_err();
    assert_eq!(
        rejected.error,
        ArtifactError::UnfinishedOwner { node: pending }
    );
    assert_eq!(rejected.parts.provenance.len(), 1);
    assert_eq!(rejected.parts.provenance[0].node, Some(pending));
    let [Op::Element(outer)] = rejected.parts.root.ops.as_slice() else {
        panic!("retained outer owner");
    };
    let [Op::Component(inner)] = outer.children.ops.as_slice() else {
        panic!("retained nested owner");
    };
    let [Op::Text(text)] = inner.children.ops.as_slice() else {
        panic!("retained nested child");
    };
    assert_eq!(text.content, "z");
    // The independent checker observes exactly the three actually minted nodes.
    assert_eq!(Artifact::try_new(*rejected.parts).unwrap().node_count(), 3);
}

fn at(source: &str, needle: &str) -> Span {
    let start = source.find(needle).unwrap() as u32;
    Span::new(start, start + needle.len() as u32)
}

#[test]
fn conditional_branches_child_values_and_spreads_seal_canonical_ops() {
    let a = Allocator::default();
    let source = "if(a){<p {...o}>{v}</p>}else{x}";
    let js =
        |text: &'static str| ExprRef::Js(JsExpr::parse_in(&a, text, at(source, text)).unwrap());
    let mut builder = Builder::new(&a, source).unwrap();
    let id = builder
        .region()
        .conditional(Span::new(0, 31), |branches, node| {
            assert_eq!(node, NodeId::FIRST);
            branches
                .branch(Some(js("a")), Span::new(0, 24), |region, _| {
                    region
                        .element(
                            "p",
                            Namespace::Html,
                            Vec::new_in(&&a),
                            at(source, "<p {...o}>{v}</p>"),
                            |region, _| {
                                assert_eq!(
                                    region
                                        .spread(js("o"), at(source, "{...o}"))
                                        .unwrap()
                                        .index(),
                                    2
                                );
                                assert_eq!(
                                    region
                                        .child_value(js("v"), at(source, "{v}"))
                                        .unwrap()
                                        .index(),
                                    3
                                );
                            },
                        )
                        .unwrap();
                })
                .unwrap();
            branches
                .branch(None, Span::new(24, 31), |region, _| {
                    assert_eq!(region.text("x", Span::new(29, 30)).unwrap().index(), 4);
                })
                .unwrap();
        })
        .unwrap();
    assert_eq!(id, NodeId::FIRST);
    let artifact = builder.finish().unwrap();
    assert_eq!(artifact.node_count(), 5);
    let [Op::If(conditional)] = artifact.root().ops.as_slice() else {
        panic!("one conditional");
    };
    let [first, second] = conditional.branches.as_slice() else {
        panic!("two branches");
    };
    assert!(first.condition.is_some() && second.condition.is_none());
    let [Op::Element(element)] = first.region.ops.as_slice() else {
        panic!("first branch element");
    };
    assert!(matches!(
        element.bindings.as_slice(),
        [crate::op::BindingOp::Bind(bind)] if bind.name.is_none()
    ));
    assert!(matches!(
        element.children.ops.as_slice(),
        [Op::JsxChildValue(_)]
    ));
    assert_eq!(
        Artifact::try_new(artifact.into_parts())
            .unwrap()
            .node_count(),
        5
    );
}

#[test]
fn conditional_shapes_and_branch_order_are_checked_as_branches_open() {
    let a = Allocator::default();
    let source = "if(a){x}else{y}";
    let condition = || ExprRef::Js(JsExpr::parse_in(&a, "a", Span::new(3, 4)).unwrap());
    let mut builder = Builder::new(&a, source).unwrap();
    builder
        .region()
        .conditional(Span::new(0, 15), |branches, node| {
            assert_eq!(
                branches.branch(None, Span::new(0, 8), |_, _| {}),
                Err(ArtifactError::InvalidIf {
                    node,
                    shape: crate::artifact::IfShape::LeadingElse
                })
            );
            branches
                .branch(Some(condition()), Span::new(0, 8), |region, _| {
                    assert!(matches!(
                        region.spread(condition(), Span::new(3, 4)),
                        Err(ArtifactError::BindingWithoutOwner { .. })
                    ));
                })
                .unwrap();
            assert!(matches!(
                branches.branch(None, Span::new(4, 15), |_, _| {}),
                Err(ArtifactError::OutsideOwner { .. })
            ));
            branches.branch(None, Span::new(8, 15), |_, _| {}).unwrap();
            assert_eq!(
                branches.branch(Some(condition()), Span::new(8, 15), |_, _| {}),
                Err(ArtifactError::InvalidIf {
                    node,
                    shape: crate::artifact::IfShape::NonTrailingElse
                })
            );
        })
        .unwrap();
    let artifact = builder.finish().unwrap();
    assert_eq!(
        Artifact::try_new(artifact.into_parts())
            .unwrap()
            .node_count(),
        1
    );

    let mut empty = Builder::new(&a, source).unwrap();
    assert_eq!(
        empty.region().conditional(Span::new(0, 15), |_, _| {}),
        Err(ArtifactError::InvalidIf {
            node: NodeId::FIRST,
            shape: crate::artifact::IfShape::Empty
        })
    );
    assert_eq!(
        empty.finish().unwrap_err().error,
        ArtifactError::UnfinishedOwner {
            node: NodeId::FIRST
        }
    );
}

/// One host statement parsed in the arena, as the TSRX host parser hands it over.
fn host_loop<'a>(a: &'a Allocator, source: &'a str) -> super::HostForHead<'a> {
    use oxc_ast::ast::Statement;
    let program = a.alloc(
        oxc_parser::Parser::new(a.as_oxc(), source, oxc_span::SourceType::mjs())
            .parse()
            .program,
    );
    let [
        Statement::ForOfStatement(head),
        Statement::VariableDeclaration(index),
        ..,
    ] = program.body.as_slice()
    else {
        panic!("expected a for-of head and an index declaration");
    };
    let oxc_ast::ast::BindingPattern::BindingIdentifier(index) = &index.declarations[0].id else {
        panic!("expected an index identifier");
    };
    let right = oxc_span::GetSpan::span(&head.right);
    super::HostForHead {
        left: &head.left,
        left_text: &source[5..24],
        collection: JsExpr::parse_in(a, "rows", Span::new(right.start, right.end)).unwrap(),
        index: Some(index),
        key: None,
        r#await: false,
    }
}

#[test]
fn host_loops_keep_the_authored_head_and_mint_empty_after_body() {
    use crate::dump::Page;
    use vize_l0::dump::{Dump, Mode};
    let a = Allocator::default();
    let source = "for (const { id, label } of rows); let i; row; none;";
    let mut builder = Builder::new(&a, source).unwrap();
    let head = host_loop(&a, source);
    let span = Span::new(0, 52);
    let id = builder
        .region()
        .host_for(head, span, |regions, owner| {
            regions
                .body(|region, _| {
                    assert_eq!(region.text("row", Span::new(42, 45)).unwrap().index(), 1);
                })
                .unwrap();
            assert_eq!(
                regions.body(|_, _| {}),
                Err(ArtifactError::InvalidHostFor { node: owner })
            );
            regions
                .empty(|region, _| {
                    assert_eq!(region.text("none", Span::new(47, 51)).unwrap().index(), 2);
                })
                .unwrap();
        })
        .unwrap();
    let artifact = builder.finish().unwrap();
    assert_eq!(artifact.node_count(), 3);
    let scope = artifact.scopes().get(id).unwrap();
    let names: alloc::vec::Vec<_> = scope.bindings.iter().map(|b| b.name.as_str()).collect();
    assert_eq!(names, ["id", "label", "i"]);
    let page = Page::of(&artifact.parts().root.ops);
    let text = page.print_to_string(Mode::Full);
    assert_eq!(
        text.as_str(),
        "[l2-dump-v2]\nops=3\n\n[l2-dump-v2.ops]\nui.for source=js(\"rows\" @28:32) \
         value=opaque(for-value \"const { id, label }\" @5:24) host-left=\"const { id, label }\" \
         host-index=\"i\" @0:52\n  ui.text \"row\" @42:45\n  empty\n    ui.text \"none\" @47:51\n\n"
    );
    assert_eq!(Page::parse(text.as_str()).unwrap(), page);

    // A host binding never carries a re-split key; the checker refuses one.
    let mut parts = artifact.into_parts();
    let Some(Op::For(for_op)) = parts.root.ops.first_mut() else {
        panic!("expected ui.for");
    };
    for_op.binding.key = Some(for_op.binding.source);
    let rejected = Artifact::try_new(parts).err().unwrap();
    assert_eq!(rejected.error, ArtifactError::InvalidHostFor { node: id });
}

#[test]
fn host_loops_need_a_body_before_their_empty_region() {
    let a = Allocator::default();
    let source = "for (const { id, label } of rows); let i; row; none;";
    let mut builder = Builder::new(&a, source).unwrap();
    let head = host_loop(&a, source);
    let result = builder
        .region()
        .host_for(head, Span::new(0, 52), |regions, owner| {
            assert_eq!(
                regions.empty(|_, _| {}),
                Err(ArtifactError::InvalidHostFor { node: owner })
            );
        });
    assert_eq!(
        result,
        Err(ArtifactError::UnfinishedOwner {
            node: NodeId::FIRST
        })
    );
}

#[test]
fn child_values_record_the_jsx_child_family() {
    let a = Allocator::default();
    let source = "{v}{{w}}";
    let mut interpolated = Builder::new(&a, source).unwrap();
    let w = JsExpr::parse_in(&a, "w", Span::new(5, 6)).unwrap();
    interpolated
        .region()
        .interpolation(ExprRef::Js(w), Span::new(3, 8))
        .unwrap();
    assert_eq!(
        interpolated.finish().unwrap().parts().features,
        LoweringFeatures::EMPTY
    );
    let mut valued = Builder::new(&a, source).unwrap();
    let v = JsExpr::parse_in(&a, "v", Span::new(1, 2)).unwrap();
    valued
        .region()
        .child_value(ExprRef::Js(v), Span::new(0, 3))
        .unwrap();
    assert_eq!(
        valued.finish().unwrap().parts().features,
        LoweringFeatures::EMPTY.observing(OpFamily::JsxChild)
    );
}
