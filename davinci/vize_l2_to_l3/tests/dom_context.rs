//! Scoped native context admission uses complete real L2 resolution tables.

use vize_l0::{Allocator, Span, id::NodeId};
use vize_l2::{
    expr::{ExprRef, JsExpr, js::JsCoordinates},
    resolution::{BindingId, BindingLookup, resolve_expression},
};
use vize_l3::decision::{
    build_dom_decisions,
    dom::{ContextOnly, DomChildren, DomExpressionFacts, DomUnsupported, ValueKind},
};

mod dom_support;
use dom_support::{SOURCE, bind, comment, element, expression, interpolation, region, seal, text};

struct ContextBindings;
impl BindingLookup for ContextBindings {
    fn lookup(&self, name: &str) -> Option<BindingId> {
        match name {
            "x" => Some(BindingId::new(1)),
            "y" => Some(BindingId::new(2)),
            _ => None,
        }
    }
}

fn js<'a>(expression: ExprRef<'a>) -> Option<&'a JsExpr<'a>> {
    match expression {
        ExprRef::Js(js) => Some(js),
        _ => None,
    }
}

fn id(index: u32) -> Option<NodeId> {
    NodeId::from_index(index)
}

#[test]
fn complete_context_refs_supply_semantics_for_text_and_ordered_properties() {
    let a = Allocator::default();
    let x = expression(&a, "x").unwrap();
    let y = expression(&a, "y").unwrap();
    let sum = expression(&a, "x + y").unwrap();
    let tables = [
        resolve_expression(js(x).expect("retained JS"), &ContextBindings).unwrap(),
        resolve_expression(js(y).expect("retained JS"), &ContextBindings).unwrap(),
        resolve_expression(js(sum).expect("retained JS"), &ContextBindings).unwrap(),
    ];
    let context = [BindingId::new(1), BindingId::new(2)];
    let policy = ContextOnly::new(&tables, &context);
    let artifact = seal(region(
        &a,
        [element(
            &a,
            [
                bind(&a, "title", x),
                bind(&a, "id", y),
                bind(&a, "class", x),
                bind(&a, "style", y),
            ],
            [text(&a), interpolation(&a, sum)],
        )],
    ))
    .unwrap();
    let analysis = build_dom_decisions(&artifact, &policy).unwrap();
    let facts = analysis.dom().unwrap();
    assert!(facts.unsupported().is_empty());
    let node = facts.node(id(0).unwrap()).unwrap();
    assert!(
        node.changes.text && node.changes.class && node.changes.style && node.changes.properties
    );
    assert_eq!(
        node.dynamic_property_bindings,
        [id(1).unwrap(), id(2).unwrap()]
    );
    assert!(
        matches!(&node.children, DomChildren::Text(group) if group.dynamic && group.nodes == [id(5).unwrap(),id(6).unwrap()])
    );
    for index in 1..=4 {
        assert_eq!(
            facts.binding(id(index).unwrap()).unwrap().value,
            ValueKind::ContextDependent
        );
    }
}

#[test]
fn missing_or_nondeclared_bindings_reject_the_whole_expression_semantics() {
    let a = Allocator::default();
    let sum = expression(&a, "x + y").unwrap();
    let tables = [resolve_expression(js(sum).expect("retained JS"), &ContextBindings).unwrap()];
    let declared = [BindingId::new(1)];
    let policy = ContextOnly::new(&tables, &declared);
    assert!(!policy.is_context_only(js(sum).expect("retained JS")));
    let artifact = seal(region(&a, [interpolation(&a, sum)])).unwrap();
    let analysis = build_dom_decisions(&artifact, &policy).unwrap();
    let rejection = analysis.dom().unwrap().unsupported().first().unwrap();
    assert_eq!(rejection.node, id(0).unwrap());
    assert_eq!(rejection.span, sum.span());
    assert_eq!(rejection.reason, DomUnsupported::Expression);
    assert!(
        !ContextOnly::new(&[], &[BindingId::new(1), BindingId::new(2)])
            .is_context_only(js(sum).expect("retained JS"))
    );
}

#[test]
fn equal_source_with_another_retained_ast_never_borrows_resolution_facts() {
    let a = Allocator::default();
    let first = expression(&a, "x").unwrap();
    let second = expression(&a, "x").unwrap();
    let tables = [resolve_expression(js(first).expect("retained JS"), &ContextBindings).unwrap()];
    let declared = [BindingId::new(1)];
    let policy = ContextOnly::new(&tables, &declared);
    assert!(policy.is_context_only(js(first).expect("retained JS")));
    assert!(!policy.is_context_only(js(second).expect("retained JS")));
    let changed = JsExpr {
        span: Span::new(2, 3),
        ..*js(first).expect("retained JS")
    };
    assert!(!policy.is_context_only(&changed));
}

#[test]
fn coordinate_bridges_must_be_the_exact_retained_identity() {
    let a = Allocator::default();
    let expression = expression(&a, "x").unwrap();
    let original = js(expression).expect("retained JS");
    let bridge = JsCoordinates::checked(SOURCE, "x", original.span, 0, &[]).unwrap();
    let retained = JsExpr::from_retained_in(&a, original.ast, "x", original.span, bridge).unwrap();
    let tables = [resolve_expression(retained, &ContextBindings).unwrap()];
    let declared = [BindingId::new(1)];
    let policy = ContextOnly::new(&tables, &declared);
    assert!(policy.is_context_only(retained));
    assert!(!policy.is_context_only(original));
    let another_bridge = JsCoordinates::checked(SOURCE, "x", original.span, 0, &[]).unwrap();
    let changed = JsExpr {
        coordinates: Some(&another_bridge),
        ..*retained
    };
    assert!(!policy.is_context_only(&changed));
}

#[test]
fn zero_reference_nonliterals_do_not_gain_const_or_dynamic_proof() {
    let a = Allocator::default();
    let zero = JsExpr::parse_in(&a, "1 + 2", Span::new(0, 5)).unwrap();
    let tables = [resolve_expression(zero, &ContextBindings).unwrap()];
    assert!(tables.first().unwrap().occurrences().is_empty());
    assert!(!ContextOnly::new(&tables, &[]).is_context_only(zero));
    let literal = expression(&a, "1").unwrap();
    let tables = [resolve_expression(js(literal).expect("retained JS"), &ContextBindings).unwrap()];
    let policy = ContextOnly::new(&tables, &[]);
    let artifact = seal(region(&a, [interpolation(&a, literal)])).unwrap();
    let analysis = build_dom_decisions(&artifact, &policy).unwrap();
    assert!(analysis.dom().unwrap().unsupported().is_empty());
}

#[test]
fn mixed_child_text_is_dynamic_without_changing_parent_direct_text_eligibility() {
    let a = Allocator::default();
    let x = expression(&a, "x").unwrap();
    let tables = [resolve_expression(js(x).expect("retained JS"), &ContextBindings).unwrap()];
    let declared = [BindingId::new(1)];
    let policy = ContextOnly::new(&tables, &declared);
    let artifact = seal(region(
        &a,
        [element(
            &a,
            [],
            [comment(&a), element(&a, [], []), interpolation(&a, x)],
        )],
    ))
    .unwrap();
    let analysis = build_dom_decisions(&artifact, &policy).unwrap();
    let node = analysis.dom().unwrap().node(id(0).unwrap()).unwrap();
    assert!(!node.changes.text);
    assert!(matches!(&node.children, DomChildren::Array(groups)
        if matches!(groups.last(), Some(vize_l3::decision::dom::DomChild::Text(group)) if group.dynamic)));
}
