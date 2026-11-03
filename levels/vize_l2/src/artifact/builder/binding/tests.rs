extern crate std;

use crate::artifact::{Artifact, ArtifactError, Builder};
use crate::expr::{ExprRef, JsExpr};
use crate::op::{BindingOp, DynamicName, Namespace, Op};
use crate::walk::{NodeEvent, NodeRef};
use vize_l0::{Allocator, Span, Vec, id::NodeId};

const SOURCE: &str = "<x :a=\"v\" :b=\"w\">t</x>";
const OWNER: Span = Span::new(0, 22);

#[test]
fn attached_bindings_are_numbered_before_children_and_retain_the_ast() {
    let a = Allocator::default();
    let mut builder = Builder::new(&a, SOURCE).unwrap();
    let value = JsExpr::parse_in(&a, "v", Span::new(7, 8)).unwrap();
    let other = JsExpr::parse_in(&a, "w", Span::new(14, 15)).unwrap();
    builder
        .element("x", Namespace::Html, Vec::new_in(&&a), OWNER, |r, owner| {
            assert_eq!(owner.index(), 0);
            assert_eq!(
                r.bind("a", Span::new(4, 5), ExprRef::Js(value), Span::new(3, 9))
                    .unwrap()
                    .index(),
                1
            );
            assert_eq!(
                r.bind(
                    "b",
                    Span::new(11, 12),
                    ExprRef::Js(other),
                    Span::new(10, 16)
                )
                .unwrap()
                .index(),
                2
            );
            assert_eq!(r.text("t", Span::new(17, 18)).unwrap().index(), 3);
        })
        .unwrap();
    let artifact = builder.finish().unwrap();
    assert_eq!(artifact.node_count(), 4);
    let mut observed = alloc::vec::Vec::new();
    artifact
        .visit_events(&mut |event| {
            if let NodeEvent::Enter {
                id,
                node,
                owner_span,
                ..
            } = event
            {
                let mnemonic = match node {
                    NodeRef::Op(op) => op.mnemonic(),
                    NodeRef::Binding(binding) => binding.mnemonic(),
                };
                observed.push((id.index(), mnemonic, owner_span));
            }
        })
        .unwrap();
    assert_eq!(
        observed,
        [
            (0, "ui.element", None),
            (1, "ui.bind", Some(OWNER)),
            (2, "ui.bind", Some(OWNER)),
            (3, "ui.text", Some(OWNER)),
        ]
    );
    let Some(Op::Element(element)) = artifact.root().ops.first() else {
        panic!("owner")
    };
    let Some(BindingOp::Bind(binding)) = element.bindings.first() else {
        panic!("binding")
    };
    assert!(matches!(binding.name, Some(DynamicName::Static("a"))));
    assert!(binding.modifiers.is_empty());
    let Some(ExprRef::Js(js)) = binding.value else {
        panic!("retained JS")
    };
    assert!(core::ptr::eq(value.ast, js.ast));
    assert_eq!(
        Artifact::try_new(artifact.into_parts())
            .unwrap()
            .node_count(),
        4
    );
}

#[test]
fn root_and_late_bindings_are_rejected_without_consuming_ids() {
    let a = Allocator::default();
    let value = JsExpr::parse_in(&a, "v", Span::new(7, 8)).unwrap();
    let mut builder = Builder::new(&a, SOURCE).unwrap();
    assert_eq!(
        builder
            .region()
            .bind("a", Span::new(4, 5), ExprRef::Js(value), Span::new(3, 9)),
        Err(ArtifactError::BindingWithoutOwner {
            span: Span::new(3, 9)
        })
    );
    builder
        .element("x", Namespace::Html, Vec::new_in(&&a), OWNER, |r, owner| {
            assert_eq!(r.text("t", Span::new(17, 18)).unwrap().index(), 1);
            assert_eq!(
                r.bind("a", Span::new(4, 5), ExprRef::Js(value), Span::new(3, 9)),
                Err(ArtifactError::BindingAfterChild {
                    node: owner,
                    span: Span::new(3, 9)
                })
            );
            assert_eq!(r.comment("", Span::new(18, 18)).unwrap().index(), 2);
        })
        .unwrap();
    assert_eq!(builder.finish().unwrap().node_count(), 3);
}

#[test]
fn names_payloads_and_failed_children_are_checked_before_phase_or_mint() {
    let a = Allocator::default();
    let value = JsExpr::parse_in(&a, "v", Span::new(7, 8)).unwrap();
    let outside = JsExpr::parse_in(&a, "w", Span::new(14, 15)).unwrap();
    let mut builder = Builder::new(&a, SOURCE).unwrap();
    builder
        .component("x", Vec::new_in(&&a), OWNER, |r, _| {
            assert!(matches!(
                r.text("", Span::new(22, 23)),
                Err(ArtifactError::InvalidSpan { .. })
            ));
            assert!(matches!(
                r.bind(
                    "wrong",
                    Span::new(4, 5),
                    ExprRef::Js(value),
                    Span::new(3, 9)
                ),
                Err(ArtifactError::InvalidBindingName { .. })
            ));
            assert!(matches!(
                r.bind("a", Span::new(4, 5), ExprRef::Js(outside), Span::new(3, 9)),
                Err(ArtifactError::OutsideOwner { .. })
            ));
            assert!(matches!(
                r.bind("a", Span::new(4, 5), ExprRef::Js(value), Span::new(0, 23)),
                Err(ArtifactError::InvalidSpan { .. })
            ));
            assert_eq!(
                r.bind("a", Span::new(4, 5), ExprRef::Js(value), Span::new(3, 9))
                    .unwrap()
                    .index(),
                1
            );
        })
        .unwrap();
    let artifact = builder.finish().unwrap();
    assert_eq!(artifact.node_count(), 2);
    assert_eq!(
        Artifact::try_new(artifact.into_parts())
            .unwrap()
            .node_count(),
        2
    );
}

#[test]
fn nested_owner_closes_parent_binding_phase() {
    let a = Allocator::default();
    let value = JsExpr::parse_in(&a, "v", Span::new(7, 8)).unwrap();
    let mut builder = Builder::new(&a, SOURCE).unwrap();
    builder
        .element("x", Namespace::Html, Vec::new_in(&&a), OWNER, |r, owner| {
            r.component("nested", Vec::new_in(&&a), Span::new(17, 18), |_, _| {})
                .unwrap();
            assert_eq!(
                r.bind("a", Span::new(4, 5), ExprRef::Js(value), Span::new(3, 9)),
                Err(ArtifactError::BindingAfterChild {
                    node: owner,
                    span: Span::new(3, 9)
                })
            );
        })
        .unwrap();
    assert_eq!(builder.finish().unwrap().node_count(), 2);
}

#[test]
fn caught_unwind_preserves_already_attached_bindings_in_the_partial_owner() {
    let a = Allocator::default();
    let value = JsExpr::parse_in(&a, "v", Span::new(7, 8)).unwrap();
    let mut builder = Builder::new(&a, SOURCE).unwrap();
    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = builder.element("x", Namespace::Html, Vec::new_in(&&a), OWNER, |r, _| {
            r.bind("a", Span::new(4, 5), ExprRef::Js(value), Span::new(3, 9))
                .unwrap();
            panic!("producer callback");
        });
    }));
    assert!(failure.is_err());
    let rejected = builder.finish().unwrap_err();
    assert_eq!(
        rejected.error,
        ArtifactError::UnfinishedOwner {
            node: NodeId::FIRST
        }
    );
    let partial = Artifact::try_new(*rejected.parts).unwrap();
    assert_eq!(partial.node_count(), 2);
    let Some(Op::Element(element)) = partial.root().ops.first() else {
        panic!("owner")
    };
    assert!(matches!(element.bindings.first(), Some(BindingOp::Bind(_))));
}
