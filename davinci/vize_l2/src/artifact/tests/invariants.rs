use super::{
    Allocator, Artifact, ArtifactError, ArtifactParts, Box, ElementOp, ExprRef, ForBinding, ForOp,
    IfBranch, IfOp, IfShape, InterpolationOp, JsExpr, Namespace, NodeId, Op, PageWalk,
    ProvenanceRecord, ScopeTag, Span, String, Vec, opaque, parts, region, scope, text,
};
use alloc::vec;

#[test]
fn source_spans_reject_inversion_outside_and_utf8_cuts() {
    let a = Allocator::default();
    for span in [Span::new(3, 1), Span::new(0, 4), Span::new(1, 2)] {
        let rejected = Artifact::try_new(parts("éx", region(&a, [text(&a, span)]))).unwrap_err();
        assert_eq!(
            rejected.error,
            ArtifactError::InvalidSpan {
                node: Some(NodeId::FIRST),
                span
            }
        );
        assert_eq!(rejected.parts.root.ops.len(), 1);
    }
    let artifact = Artifact::try_new(parts("éx", region(&a, [text(&a, Span::new(2, 3))]))).unwrap();
    assert_eq!(artifact.node_count(), 1);
}

#[test]
fn child_and_expression_ranges_must_stay_in_their_source_owner() {
    let a = Allocator::default();
    let root = region(
        &a,
        [Op::Interpolation(Box::new_in(
            InterpolationOp {
                expression: opaque(&a, Span::new(1, 3)),
                span: Span::new(0, 2),
            },
            &&a,
        ))],
    );
    assert_eq!(
        Artifact::try_new(parts("abc", root)).unwrap_err().error,
        ArtifactError::OutsideOwner {
            node: NodeId::FIRST,
            span: Span::new(1, 3),
            owner: Span::new(0, 2)
        }
    );
    let root = region(
        &a,
        [Op::Element(Box::new_in(
            ElementOp {
                tag: "p",
                namespace: Namespace::Html,
                attributes: Vec::new_in(&&a),
                bindings: Vec::new_in(&&a),
                children: region(&a, [text(&a, Span::new(1, 3))]),
                span: Span::new(0, 2),
            },
            &&a,
        ))],
    );
    assert!(matches!(
        Artifact::try_new(parts("abc", root)).unwrap_err().error,
        ArtifactError::OutsideOwner { .. }
    ));
}

#[test]
fn canonical_if_shapes_are_checked_on_native_branches() {
    let a = Allocator::default();
    let span = Span::new(0, 1);
    for (conditions, expected) in [
        (vec![], IfShape::Empty),
        (vec![false], IfShape::LeadingElse),
        (vec![true, false, true], IfShape::NonTrailingElse),
    ] {
        let mut branches = Vec::new_in(&&a);
        for condition in conditions {
            branches.push(IfBranch {
                condition: condition.then(|| opaque(&a, span)),
                region: region(&a, []),
                span,
            });
        }
        let root = region(&a, [Op::If(Box::new_in(IfOp { branches, span }, &&a))]);
        assert_eq!(
            Artifact::try_new(parts("x", root)).unwrap_err().error,
            ArtifactError::InvalidIf {
                node: NodeId::FIRST,
                shape: expected
            }
        );
    }
}

#[test]
fn stale_provenance_and_non_introducing_scope_keys_are_rejected() {
    let a = Allocator::default();
    let span = Span::new(0, 1);
    let mut input = parts("x", region(&a, [text(&a, span)]));
    let stale = NodeId::from_index(1).unwrap();
    input.provenance.push(ProvenanceRecord {
        rule: String::from("test"),
        node: Some(stale),
        before: String::from("x"),
        after: String::from("ui.text"),
        span,
    });
    assert_eq!(
        Artifact::try_new(input).unwrap_err().error,
        ArtifactError::DanglingProvenance {
            record: 0,
            node: stale
        }
    );
    let mut input = parts("x", region(&a, [text(&a, span)]));
    input.scopes.insert(NodeId::FIRST, scope(0));
    assert_eq!(
        Artifact::try_new(input).unwrap_err().error,
        ArtifactError::InvalidScopeSite {
            node: NodeId::FIRST
        }
    );
}

fn loops<'a>(a: &'a Allocator) -> ArtifactParts<'a> {
    let span = Span::new(0, 1);
    let mut root = region(a, []);
    for _ in 0..2 {
        root.ops.push(Op::For(Box::new_in(
            ForOp {
                binding: ForBinding {
                    source: opaque(a, span),
                    value: opaque(a, span),
                    key: None,
                    index: None,
                },
                region: region(a, []),
                span,
            },
            &a,
        )));
    }
    parts("x", root)
}

#[test]
fn every_scope_site_has_a_unique_dense_tag_even_without_enumerated_names() {
    let a = Allocator::default();
    let second = NodeId::from_index(1).unwrap();
    assert_eq!(
        Artifact::try_new(loops(&a)).unwrap_err().error,
        ArtifactError::MissingScope {
            node: NodeId::FIRST
        }
    );
    for tag in [0, 2] {
        let mut input = loops(&a);
        input.scopes.insert(NodeId::FIRST, scope(0));
        input.scopes.insert(second, scope(tag));
        assert_eq!(
            Artifact::try_new(input).unwrap_err().error,
            ArtifactError::InvalidScopeTag {
                node: second,
                tag: ScopeTag::from_index(tag)
            }
        );
    }
    let mut input = loops(&a);
    input.scopes.insert(second, scope(1));
    input.scopes.insert(NodeId::FIRST, scope(0));
    assert_eq!(Artifact::try_new(input).unwrap().node_count(), 2);
}

#[test]
fn retained_js_root_uses_its_own_text_coordinates_without_reparsing() {
    let a = Allocator::default();
    let parsed = JsExpr::parse_in(&a, "longName", Span::new(0, 1)).unwrap();
    let malformed = a.alloc(JsExpr {
        ast: parsed.ast,
        source: "x",
        span: Span::new(0, 1),
        coordinates: None,
    });
    let root = region(
        &a,
        [Op::Interpolation(Box::new_in(
            InterpolationOp {
                expression: ExprRef::Js(malformed),
                span: Span::new(0, 1),
            },
            &&a,
        ))],
    );
    assert_eq!(
        Artifact::try_new(parts("x", root)).unwrap_err().error,
        ArtifactError::InvalidJsSpan {
            node: NodeId::FIRST,
            span: Span::new(0, 8)
        }
    );
}

#[test]
fn skipped_attached_ids_do_not_dispatch_extra_region_op_visits() {
    let mut by_mint = PageWalk::new();
    let mut by_skip = PageWalk::new();
    assert_eq!(by_mint.mint(), by_skip.mint());
    assert_eq!(by_mint.mint_attached().unwrap().index(), 1);
    assert_eq!(by_mint.mint_attached().unwrap().index(), 2);
    by_skip.skip(2);
    assert_eq!(by_mint.mint(), by_skip.mint());
    assert_eq!(by_mint.visits(), 2);
    assert_eq!(by_skip.visits(), 2);
    assert_eq!(by_mint.minted(), by_skip.minted());
}

#[test]
fn id_exhaustion_is_reported_without_wrapping_or_unbalanced_fake_nodes() {
    let a = Allocator::default();
    let root = region(&a, [text(&a, Span::new(0, 1))]);
    let mut walk = PageWalk::new();
    walk.skip(u32::MAX as usize);
    let mut events = 0;
    assert_eq!(
        crate::walk::visit_events(&mut walk, &root.ops, &mut |_| events += 1),
        Err(crate::walk::NodeLimit)
    );
    assert_eq!(events, 0);
    assert!(walk.is_exhausted());
}
