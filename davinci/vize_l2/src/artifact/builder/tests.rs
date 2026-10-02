extern crate std;

use super::Builder;
use crate::artifact::{Artifact, ArtifactError};
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
