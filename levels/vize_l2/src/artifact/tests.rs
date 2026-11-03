use alloc::vec;
use alloc::vec::Vec as OwnedVec;
use vize_l0::{Allocator, Box, Span, String, Vec, id::NodeId, side_table::SideTable};

use super::{Artifact, ArtifactError, ArtifactParts, IfShape};
use crate::expr::{ExprRef, JsExpr, OpaqueExpr, OpaqueReason};
use crate::op::{
    BindingOp, ComponentOp, DynamicName, ElementOp, ForBinding, ForOp, IfBranch, IfOp,
    InterpolationOp, Namespace, Op, Region, SlotContentOp, SlotOp, TextOp, VueOnceOp,
};
use crate::provenance::ProvenanceRecord;
use crate::scope::{ScopeBinding, ScopeFacts, ScopeOrigin, ScopeTag};
use crate::walk::{NodeEvent, NodeRef, PageWalk};

mod invariants;

fn region<'a>(a: &'a Allocator, ops: impl IntoIterator<Item = Op<'a>>) -> Region<'a> {
    let mut result = Region {
        ops: Vec::new_in(&a),
    };
    result.ops.extend(ops);
    result
}

fn text<'a>(a: &'a Allocator, span: Span) -> Op<'a> {
    Op::Text(Box::new_in(TextOp { content: "x", span }, &a))
}

fn opaque<'a>(a: &'a Allocator, span: Span) -> ExprRef<'a> {
    ExprRef::Opaque(a.alloc(OpaqueExpr {
        reason: OpaqueReason::ParseRejected,
        source: "?",
        span,
    }))
}

fn parts<'a>(source: &'a str, root: Region<'a>) -> ArtifactParts<'a> {
    ArtifactParts {
        source,
        root,
        provenance: OwnedVec::new(),
        scopes: SideTable::new(),
    }
}

fn scope(tag: u32) -> ScopeFacts {
    ScopeFacts {
        tag: ScopeTag::from_index(tag),
        bindings: OwnedVec::new(),
    }
}

#[test]
fn retained_js_carrier_and_ast_are_the_original_once_parsed_payload() {
    let a = Allocator::default();
    let js = JsExpr::parse_in(&a, "value", Span::new(2, 7)).unwrap();
    let root = region(
        &a,
        [Op::Interpolation(Box::new_in(
            InterpolationOp {
                expression: ExprRef::Js(js),
                span: Span::new(0, 9),
            },
            &&a,
        ))],
    );
    let artifact = Artifact::try_new(parts("{{value}}", root)).unwrap();
    let mut seen = 0;
    artifact
        .visit_nodes(&mut |id, node| {
            assert_eq!(id, NodeId::FIRST);
            node.for_each_expression(&mut |expr| {
                let ExprRef::Js(retained) = expr else {
                    panic!("native JS carrier must stay JS")
                };
                assert!(core::ptr::eq(retained, js));
                assert!(core::ptr::eq(retained.ast, js.ast));
                assert!(core::ptr::eq(retained.source.as_ptr(), js.source.as_ptr()));
                assert_eq!(retained.span, Span::new(2, 7));
                seen += 1;
            });
        })
        .unwrap();
    assert_eq!(seen, 1);
}

#[test]
fn attached_ids_and_mutable_pass_accounting_share_exact_page_order() {
    let a = Allocator::default();
    let span = Span::new(0, 1);
    let mut bindings = Vec::new_in(&&a);
    bindings.push(BindingOp::VueOnce(Box::new_in(VueOnceOp { span }, &&a)));
    let mut slot_bindings = Vec::new_in(&&a);
    slot_bindings.push(BindingOp::SlotContent(Box::new_in(
        SlotContentOp {
            name: None,
            modifiers: Vec::new_in(&&a),
            params: Some(opaque(&a, span)),
            span,
        },
        &&a,
    )));
    let slot = Op::Slot(Box::new_in(
        SlotOp {
            name: DynamicName::Static("default"),
            attributes: Vec::new_in(&&a),
            bindings: slot_bindings,
            fallback: region(&a, [text(&a, span)]),
            span,
        },
        &&a,
    ));
    let for_op = Op::For(Box::new_in(
        ForOp {
            binding: ForBinding {
                source: opaque(&a, span),
                value: opaque(&a, span),
                key: None,
                index: None,
            },
            region: region(&a, [slot]),
            span,
        },
        &&a,
    ));
    let mut branches = Vec::new_in(&&a);
    branches.push(IfBranch {
        condition: Some(opaque(&a, span)),
        region: region(&a, [for_op]),
        span,
    });
    branches.push(IfBranch {
        condition: None,
        region: region(&a, [text(&a, span)]),
        span,
    });
    let conditional = Op::If(Box::new_in(IfOp { branches, span }, &&a));
    let component = Op::Component(Box::new_in(
        ComponentOp {
            name: "C",
            attributes: Vec::new_in(&&a),
            bindings: Vec::new_in(&&a),
            children: region(&a, [text(&a, span)]),
            span,
        },
        &&a,
    ));
    let element = Op::Element(Box::new_in(
        ElementOp {
            tag: "div",
            namespace: Namespace::Html,
            attributes: Vec::new_in(&&a),
            bindings,
            children: region(&a, [conditional, component]),
            span,
        },
        &&a,
    ));
    let mut input = parts("x", region(&a, [element, text(&a, span)]));
    input
        .scopes
        .insert(NodeId::from_index(3).unwrap(), scope(0));
    input
        .scopes
        .insert(NodeId::from_index(5).unwrap(), scope(1));
    let mut old = OwnedVec::new();
    let mut walk = PageWalk::new();
    crate::walk::visit_ops(&mut walk, &mut input.root.ops, &mut |id, op| {
        old.push((id.unwrap().index(), op.mnemonic()));
    });
    assert_eq!(walk.minted(), 11);
    assert_eq!(walk.visits(), 9);
    let artifact = Artifact::try_new(input).unwrap();
    assert_eq!(artifact.node_count(), 11);
    let mut all = OwnedVec::new();
    let mut ops = OwnedVec::new();
    artifact
        .visit_nodes(&mut |id, node| {
            all.push((id.index(), node.mnemonic()));
            if matches!(node, NodeRef::Op(_)) {
                ops.push((id.index(), node.mnemonic()));
            }
        })
        .unwrap();
    assert_eq!(
        all,
        vec![
            (0, "ui.element"),
            (1, "vue.once"),
            (2, "ui.if"),
            (3, "ui.for"),
            (4, "ui.slot"),
            (5, "ui.slot-content"),
            (6, "ui.text"),
            (7, "ui.text"),
            (8, "ui.component"),
            (9, "ui.text"),
            (10, "ui.text"),
        ]
    );
    assert_eq!(ops, old);
    let mut stack = OwnedVec::new();
    let mut enters = OwnedVec::new();
    let mut leaves = OwnedVec::new();
    artifact
        .visit_events(&mut |event| match event {
            NodeEvent::Enter { id, parent, .. } => {
                assert_eq!(parent, stack.last().copied());
                stack.push(id);
                enters.push(id.index());
            }
            NodeEvent::Leave { id, parent, .. } => {
                assert_eq!(stack.pop(), Some(id));
                assert_eq!(parent, stack.last().copied());
                leaves.push(id.index());
            }
        })
        .unwrap();
    assert!(stack.is_empty());
    assert_eq!(enters, (0..11).collect::<OwnedVec<_>>());
    assert_eq!(leaves, vec![1, 5, 6, 4, 3, 7, 2, 9, 8, 0, 10]);
}

#[test]
fn scope_origin_order_and_failed_provenance_survive_sealing_and_rejection() {
    let a = Allocator::default();
    let span = Span::new(0, 1);
    let mut root = region(&a, []);
    root.ops.push(Op::For(Box::new_in(
        ForOp {
            binding: ForBinding {
                source: opaque(&a, span),
                value: opaque(&a, span),
                key: None,
                index: None,
            },
            region: region(&a, [text(&a, span)]),
            span,
        },
        &&a,
    )));
    let mut input = parts("x", root);
    input.scopes.insert(
        NodeId::FIRST,
        ScopeFacts {
            tag: ScopeTag::from_index(0),
            bindings: vec![
                ScopeBinding {
                    name: String::from("x"),
                    origin: ScopeOrigin::Authored { span },
                },
                ScopeBinding {
                    name: String::from("x"),
                    origin: ScopeOrigin::Synthesized {
                        rule: String::from("test.synthesis"),
                    },
                },
            ],
        },
    );
    input.provenance.push(ProvenanceRecord {
        rule: String::from("error.test"),
        node: None,
        before: String::from("x"),
        after: String::default(),
        span,
    });
    let artifact = Artifact::try_new(input).unwrap();
    assert_eq!(artifact.provenance()[0].node, None);
    assert_eq!(
        artifact.scopes().get(NodeId::FIRST).unwrap().bindings.len(),
        2
    );
    let mut changed = artifact.into_parts();
    changed.root.ops.clear();
    let rejected = Artifact::try_new(changed).unwrap_err();
    assert_eq!(
        rejected.error,
        ArtifactError::DanglingScope {
            node: NodeId::FIRST
        }
    );
    assert_eq!(rejected.parts.provenance[0].rule.as_str(), "error.test");
    assert!(rejected.parts.provenance[0].after.is_empty());
    assert!(rejected.parts.scopes.contains_id(NodeId::FIRST));
}

#[test]
fn empty_artifact_has_no_numbered_nodes_and_preserves_source_identity() {
    let a = Allocator::default();
    let artifact = Artifact::try_new(parts("source", region(&a, []))).unwrap();
    assert_eq!(artifact.node_count(), 0);
    assert!(!artifact.contains_node(NodeId::FIRST));
    let mut calls = 0;
    artifact.visit_events(&mut |_| calls += 1).unwrap();
    assert_eq!(calls, 0);
    assert_eq!(artifact.source(), "source");
    assert!(artifact.root().ops.is_empty());
}
