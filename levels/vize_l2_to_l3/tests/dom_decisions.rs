//! Real owner-bound DOM facts, without a legacy or fabricated semantic input.

use vize_l0::{Allocator, id::NodeId};
use vize_l2::op::{Attribute, Namespace, Op};
use vize_l3::decision::{
    build_decisions,
    dom::{DomChild, DomChildren, DomRootKind, DomText, DomUnsupported, PropertyRole, ValueKind},
    policy::TargetPolicy,
};

mod dom_support;
use dom_support::{SPAN, bind, comment, element, expression, interpolation, region, seal, text};

fn id(index: u32) -> Option<NodeId> {
    NodeId::from_index(index)
}

#[test]
fn facts_are_dom_only_and_retain_exact_canonical_payloads() {
    let a = Allocator::default();
    let artifact = seal(region(&a, [element(&a, [], [text(&a)])])).unwrap();
    for policy in [TargetPolicy::Ssr, TargetPolicy::Vapor] {
        assert!(build_decisions(&artifact, policy).unwrap().dom().is_none());
    }
    let analysis = build_decisions(&artifact, TargetPolicy::Dom).unwrap();
    let facts = analysis.dom().unwrap();
    assert!(facts.unsupported().is_empty());
    let first = artifact.root().ops.first().unwrap();
    assert!(core::ptr::eq(
        facts.node(id(0).unwrap()).unwrap().op(),
        first
    ));
    assert!(facts.node(id(0).unwrap()).unwrap().block_eligible);
    assert_eq!(facts.root().kind, DomRootKind::Direct);
    assert!(facts.node(id(2).unwrap()).is_none());
}

#[test]
fn literal_text_groups_have_no_runtime_change_eligibility() {
    let a = Allocator::default();
    let literal = expression(&a, "1").unwrap();
    let artifact = seal(region(
        &a,
        [element(
            &a,
            [],
            [text(&a), interpolation(&a, literal), text(&a)],
        )],
    ))
    .unwrap();
    let analysis = build_decisions(&artifact, TargetPolicy::Dom).unwrap();
    let facts = analysis.dom().unwrap();
    assert!(facts.unsupported().is_empty());
    assert_eq!(
        facts.node(id(0).unwrap()).unwrap().children,
        DomChildren::Text(DomText {
            nodes: vec![id(1).unwrap(), id(2).unwrap(), id(3).unwrap()],
            dynamic: false
        })
    );
    assert!(!facts.node(id(0).unwrap()).unwrap().changes.text);
}

#[test]
fn mixed_children_preserve_contiguous_groups_and_nested_nonblocks() {
    let a = Allocator::default();
    let literal = expression(&a, "1").unwrap();
    let artifact = seal(region(
        &a,
        [element(
            &a,
            [],
            [
                text(&a),
                element(&a, [], []),
                interpolation(&a, literal),
                text(&a),
                comment(&a),
            ],
        )],
    ))
    .unwrap();
    let analysis = build_decisions(&artifact, TargetPolicy::Dom).unwrap();
    let facts = analysis.dom().unwrap();
    assert_eq!(
        facts.node(id(0).unwrap()).unwrap().children,
        DomChildren::Array(vec![
            DomChild::Text(DomText {
                nodes: vec![id(1).unwrap()],
                dynamic: false
            }),
            DomChild::Node(id(2).unwrap()),
            DomChild::Text(DomText {
                nodes: vec![id(3).unwrap(), id(4).unwrap()],
                dynamic: false
            }),
            DomChild::Node(id(5).unwrap()),
        ])
    );
    assert!(!facts.node(id(2).unwrap()).unwrap().block_eligible);
    assert!(!facts.node(id(0).unwrap()).unwrap().changes.text);
}

#[test]
fn root_groups_and_comment_count_choose_semantic_fragment_eligibility() {
    let a = Allocator::default();
    let empty = seal(region(&a, [])).unwrap();
    let analysis = build_decisions(&empty, TargetPolicy::Dom).unwrap();
    assert_eq!(analysis.dom().unwrap().root().kind, DomRootKind::Empty);
    let artifact = seal(region(&a, [comment(&a), element(&a, [], [])])).unwrap();
    let analysis = build_decisions(&artifact, TargetPolicy::Dom).unwrap();
    let facts = analysis.dom().unwrap();
    assert_eq!(
        facts.root().kind,
        DomRootKind::Fragment {
            single_non_comment: true
        }
    );
    assert!(!facts.node(id(1).unwrap()).unwrap().block_eligible);
    let artifact = seal(region(&a, [element(&a, [], []), text(&a), comment(&a)])).unwrap();
    let analysis = build_decisions(&artifact, TargetPolicy::Dom).unwrap();
    assert_eq!(
        analysis.dom().unwrap().root().kind,
        DomRootKind::Fragment {
            single_non_comment: false
        }
    );
}

#[test]
fn literal_bindings_keep_actual_ids_roles_and_payloads_without_dynamic_props() {
    let a = Allocator::default();
    let number = expression(&a, "1").unwrap();
    let string = expression(&a, "'a'").unwrap();
    let artifact = seal(region(
        &a,
        [element(
            &a,
            [
                bind(&a, "id", number),
                bind(&a, "class", string),
                bind(&a, "style", string),
            ],
            [],
        )],
    ))
    .unwrap();
    let analysis = build_decisions(&artifact, TargetPolicy::Dom).unwrap();
    let facts = analysis.dom().unwrap();
    assert!(facts.unsupported().is_empty());
    let canonical = artifact.root().ops.first().unwrap();
    assert!(matches!(canonical, Op::Element(_)));
    let Op::Element(element) = canonical else {
        return;
    };
    for (index, role) in [
        PropertyRole::Property,
        PropertyRole::Class,
        PropertyRole::Style,
    ]
    .into_iter()
    .enumerate()
    {
        let row = facts.binding(id(index as u32 + 1).unwrap()).unwrap();
        assert_eq!(row.role, role);
        assert_eq!(row.value, ValueKind::LiteralConstant);
        assert!(core::ptr::eq(
            row.binding(),
            element.bindings.get(index).unwrap()
        ));
    }
    assert_eq!(
        facts.node(id(0).unwrap()).unwrap().changes,
        Default::default()
    );
    assert!(
        facts
            .node(id(0).unwrap())
            .unwrap()
            .dynamic_property_bindings
            .is_empty()
    );
}

#[test]
fn missing_context_semantics_are_explicit_and_never_forged_dynamic_flags() {
    let a = Allocator::default();
    let identifier = expression(&a, "x").unwrap();
    let artifact = seal(region(
        &a,
        [element(
            &a,
            [bind(&a, "id", identifier)],
            [interpolation(&a, identifier)],
        )],
    ))
    .unwrap();
    let analysis = build_decisions(&artifact, TargetPolicy::Dom).unwrap();
    let facts = analysis.dom().unwrap();
    assert_eq!(
        facts
            .unsupported()
            .iter()
            .map(|r| (r.node, r.reason))
            .collect::<Vec<_>>(),
        [
            (id(1).unwrap(), DomUnsupported::Expression),
            (id(2).unwrap(), DomUnsupported::Expression)
        ]
    );
    assert!(
        facts
            .unsupported()
            .iter()
            .all(|r| r.span == identifier.span())
    );
    assert!(facts.binding(id(1).unwrap()).is_none());
    assert_eq!(
        facts.node(id(0).unwrap()).unwrap().changes,
        Default::default()
    );
}

#[test]
fn namespaces_special_attributes_and_duplicate_properties_stay_unsupported() {
    let a = Allocator::default();
    let literal = expression(&a, "1").unwrap();
    let mut native = element(&a, [bind(&a, "id", literal), bind(&a, "id", literal)], []);
    assert!(matches!(native, Op::Element(_)));
    let Op::Element(element) = &mut native else {
        return;
    };
    element.namespace = Namespace::Svg;
    element.attributes.push(Attribute {
        name: "style",
        value: Some("color:red"),
        span: SPAN,
    });
    let artifact = seal(region(&a, [native])).unwrap();
    let analysis = build_decisions(&artifact, TargetPolicy::Dom).unwrap();
    assert_eq!(
        analysis
            .dom()
            .unwrap()
            .unsupported()
            .iter()
            .map(|r| (r.node, r.reason))
            .collect::<Vec<_>>(),
        [
            (id(0).unwrap(), DomUnsupported::Namespace),
            (id(0).unwrap(), DomUnsupported::SpecialAttribute),
            (id(2).unwrap(), DomUnsupported::DuplicateProperty),
        ]
    );
}
