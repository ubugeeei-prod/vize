extern crate std;

use super::ConditionalFrame;
use crate::decision::DecisionBuildError;
use alloc::vec::Vec as Owned;
use vize_l0::{Allocator, Span, Vec, id::NodeId};
use vize_l2::{
    artifact::{Artifact, Builder, ConditionalBranch},
    expr::{ExprRef, JsExpr},
    op::{Namespace, Op},
    walk::{NodeEvent, NodeRef},
};

fn fixture(a: &Allocator, first_roots: usize) -> Artifact<'_> {
    let source = "ok";
    let span = Span::new(0, 2);
    let condition = ExprRef::Js(JsExpr::parse_in(a, source, span).unwrap());
    let mut builder = Builder::new(a, source).unwrap();
    builder
        .conditional(
            &[
                ConditionalBranch {
                    condition: Some(condition),
                    span,
                },
                ConditionalBranch {
                    condition: None,
                    span,
                },
            ],
            span,
            |region, _, index| {
                for _ in 0..if index == 0 { first_roots } else { 1 } {
                    region
                        .element("p", Namespace::Html, Vec::new_in(&a), span, |_, _| {})
                        .unwrap();
                }
            },
        )
        .unwrap();
    builder.finish().unwrap()
}

#[test]
fn incomplete_direct_children_cannot_publish_a_completed_control() {
    let a = Allocator::default();
    let artifact = fixture(&a, 1);
    let Op::If(owner) = artifact.root().ops.first().unwrap() else {
        panic!("If")
    };
    let mut frame = ConditionalFrame::new(owner, true);
    let mut first = None;
    artifact
        .visit_events(&mut |event| {
            if first.is_none()
                && let NodeEvent::Enter {
                    id,
                    node: NodeRef::Op(op),
                    parent: Some(parent),
                    owner_span,
                } = event
                && parent == NodeId::FIRST
            {
                first = Some(frame.child(id, op, owner_span));
            }
        })
        .unwrap();
    assert!(first.unwrap().unwrap());
    assert!(matches!(frame.finish(NodeId::FIRST),
        Err(DecisionBuildError::InvalidTraversal { node }) if node == NodeId::FIRST));
}

#[test]
fn wrong_header_owner_span_is_a_typed_invariant_error() {
    let a = Allocator::default();
    let artifact = fixture(&a, 1);
    let Op::If(owner) = artifact.root().ops.first().unwrap() else {
        panic!("If")
    };
    let mut frame = ConditionalFrame::new(owner, true);
    let mut error = None;
    artifact
        .visit_events(&mut |event| {
            if error.is_none()
                && let NodeEvent::Enter {
                    id,
                    node: NodeRef::Op(op),
                    parent: Some(parent),
                    ..
                } = event
                && parent == NodeId::FIRST
            {
                error = Some(frame.child(id, op, Some(Span::new(0, 1))));
            }
        })
        .unwrap();
    assert!(
        matches!(error.unwrap(), Err(DecisionBuildError::InvalidTraversal { node })
        if node.index() == 1)
    );
}

#[test]
fn skipped_empty_header_does_not_consume_the_next_actual_root() {
    let a = Allocator::default();
    let artifact = fixture(&a, 0);
    let Op::If(owner) = artifact.root().ops.first().unwrap() else {
        panic!("If")
    };
    let mut frame = ConditionalFrame::new(owner, false);
    let mut observed = Owned::new();
    artifact
        .visit_events(&mut |event| {
            if let NodeEvent::Enter {
                id,
                node: NodeRef::Op(op),
                parent: Some(parent),
                owner_span,
            } = event
                && parent == NodeId::FIRST
            {
                observed.push(frame.child(id, op, owner_span));
            }
        })
        .unwrap();
    assert_eq!(observed.len(), 1);
    assert!(observed.first().unwrap().is_ok());
    assert!(frame.finish(NodeId::FIRST).unwrap().is_none());
}
