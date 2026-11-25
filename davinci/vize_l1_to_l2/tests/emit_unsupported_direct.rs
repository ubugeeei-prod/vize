//! Direct L2 refusal fixtures for malformed or fact-incomplete handoffs.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "tests assert by panicking"
)]

use std::vec::Vec as StdVec;

use vize_l0::id::NodeId;
use vize_l0::{Allocator, Box as ArenaBox, Span, String as VString, Vec as ArenaVec};
use vize_l1_to_l2::pass::{SlotAnalysis, SlotCarrier, SlotGroup, SlotName, SlotParams};
use vize_l1_to_l2::{LegacyCaps, Lowered};
use vize_l2::analysis::fragment_facts::{FragmentKey, LoopFragment};
use vize_l2::analysis::pass_facts::L2Analyses;
use vize_l2::analysis::text_facts::TextParts;
use vize_l2::artifact::L2TemplateParts;
use vize_l2::expr::{ExprRef, ForeignExpr, OpaqueExpr, OpaqueReason};
use vize_l2::op::{
    Attribute, BindingContract, BindingOp, ComponentOp, ElementOp, ForBinding, ForOp, HostFor,
    IfBranch, IfOp, InterpolationOp, ModelOp, Namespace, Op, Region, TextOp,
};
use vize_l2::scope::ScopeOrigin;
use vize_l4::targets::dom::artifact::{EmitError, UnsupportedReason as Reason, emit_dom};

type Factory = for<'a> fn(&'a Allocator) -> (Lowered<'a>, L2Analyses);

const DIRECT: &[(Reason, Factory)] = &[
    (Reason::EmptyCompoundText, empty_compound_text),
    (Reason::ForAliasNotEmittable, for_alias_not_emittable),
    (Reason::ForItemShape, for_item_shape),
    (Reason::HostForLoop, host_for_loop),
    (Reason::IfBranchShape, if_branch_shape),
    (Reason::IfWithoutBranches, if_without_branches),
    (Reason::MissingTextFacts, missing_text_facts),
    (Reason::ModelExpressionNotJs, model_expression_not_js),
    (Reason::SlotFactsMissingGroup, slot_facts_missing_group),
    (Reason::TemplateDynamicKeyEmpty, template_dynamic_key_empty),
];

#[test]
fn direct_l2_refusal_fixtures_are_classified() {
    for (expected, factory) in DIRECT {
        let allocator = Allocator::new();
        let (lowered, facts) = factory(&allocator);
        let error = emit_dom(&lowered, &facts).expect_err(expected.code());
        assert_eq!(reason(error), *expected, "{}", expected.code());
    }
}

fn reason(error: EmitError) -> Reason {
    match error {
        EmitError::Unsupported(refusal) => refusal.reason,
        EmitError::Diagnostics => panic!("direct fixture produced diagnostics"),
    }
}

fn empty_compound_text<'a>(a: &'a Allocator) -> (Lowered<'a>, L2Analyses) {
    let mut lowered = lowered(a, root_with_compound_child(a));
    lowered.artifact.texts.insert(
        id(1),
        TextParts {
            parts: StdVec::new(),
        },
    );
    (lowered, L2Analyses::default())
}

fn missing_text_facts<'a>(a: &'a Allocator) -> (Lowered<'a>, L2Analyses) {
    (
        lowered(a, root_with_compound_child(a)),
        L2Analyses::default(),
    )
}

fn if_without_branches<'a>(a: &'a Allocator) -> (Lowered<'a>, L2Analyses) {
    let if_op = IfOp {
        branches: av(a),
        span: sp(),
    };
    (
        lowered(a, region(a, [Op::If(ab(a, if_op))])),
        L2Analyses::default(),
    )
}

fn if_branch_shape<'a>(a: &'a Allocator) -> (Lowered<'a>, L2Analyses) {
    let branch = IfBranch {
        condition: Some(js(a, "ok")),
        region: empty_region(a),
        span: sp(),
    };
    let if_op = IfOp {
        branches: av1(a, branch),
        span: sp(),
    };
    (
        lowered(a, region(a, [Op::If(ab(a, if_op))])),
        L2Analyses::default(),
    )
}

fn for_item_shape<'a>(a: &'a Allocator) -> (Lowered<'a>, L2Analyses) {
    (
        lowered(a, region(a, [for_op(a, js(a, "item"), empty_region(a))])),
        L2Analyses::default(),
    )
}

/// A host-syntax head with an `@empty` region; the DOM target has no emission for it yet.
fn host_for_loop<'a>(a: &'a Allocator) -> (Lowered<'a>, L2Analyses) {
    let source = "for (const item of items);";
    let program = a.alloc(
        oxc_parser::Parser::new(a.as_oxc(), source, oxc_span::SourceType::mjs())
            .parse()
            .program,
    );
    let Some(oxc_ast::ast::Statement::ForOfStatement(statement)) = program.body.first() else {
        panic!("expected a for-of statement");
    };
    let host = HostFor {
        left: &statement.left,
        left_text: "const item",
        index: None,
        key: None,
        r#await: false,
        empty: Some(region(a, [text(a, "none")])),
    };
    let Op::For(mut for_op) = for_op(
        a,
        opaque(a, "const item", OpaqueReason::ForValue),
        region(a, [text(a, "row")]),
    ) else {
        panic!("expected ui.for");
    };
    for_op.host = Some(ab(a, host));
    (
        lowered(a, region(a, [Op::For(for_op)])),
        L2Analyses::default(),
    )
}

fn for_alias_not_emittable<'a>(a: &'a Allocator) -> (Lowered<'a>, L2Analyses) {
    (
        lowered(
            a,
            region(a, [for_op(a, foreign(a, "pattern"), empty_region(a))]),
        ),
        L2Analyses::default(),
    )
}

fn model_expression_not_js<'a>(a: &'a Allocator) -> (Lowered<'a>, L2Analyses) {
    let expr = opaque(a, "bad", OpaqueReason::ParseRejected);
    let model = BindingOp::Model(ab(
        a,
        ModelOp {
            contract: BindingContract {
                read: expr,
                write: expr,
            },
            argument: None,
            attributes: av(a),
            span: sp(),
        },
    ));
    (
        lowered(
            a,
            region(a, [el(a, "input", av1(a, model), empty_region(a))]),
        ),
        L2Analyses::default(),
    )
}

fn slot_facts_missing_group<'a>(a: &'a Allocator) -> (Lowered<'a>, L2Analyses) {
    let mut facts = L2Analyses::default();
    facts.slot_facts.insert(
        NodeId::FIRST,
        SlotAnalysis {
            groups: vec![SlotGroup {
                name: SlotName::Static {
                    text: VString::from("header"),
                    origin: ScopeOrigin::Synthesized {
                        rule: VString::from("test"),
                    },
                },
                params: SlotParams::Absent,
                carrier: SlotCarrier::Template(None),
            }],
        },
    );
    let child = text(a, "fallback");
    let root = comp(a, "Panel", av(a), region(a, [child]));
    (lowered(a, region(a, [root])), facts)
}

fn template_dynamic_key_empty<'a>(a: &'a Allocator) -> (Lowered<'a>, L2Analyses) {
    let body = region(a, [el(a, "div", av(a), empty_region(a))]);
    let mut lowered = lowered(a, region(a, [for_op(a, js(a, "item"), body)]));
    lowered.artifact.loop_fragments.insert(
        NodeId::FIRST,
        LoopFragment {
            key: Some(FragmentKey::Dynamic {
                source: VString::default(),
                span: sp(),
            }),
            attributes: StdVec::new(),
            class: None,
            runtime_directive: None,
        },
    );
    (lowered, L2Analyses::default())
}

fn root_with_compound_child<'a>(a: &'a Allocator) -> Region<'a> {
    let child = compound(a);
    region(a, [el(a, "div", av(a), region(a, [child]))])
}

fn for_op<'a>(a: &'a Allocator, value: ExprRef<'a>, body: Region<'a>) -> Op<'a> {
    Op::For(ab(
        a,
        ForOp {
            binding: ForBinding {
                source: js(a, "items"),
                value,
                key: None,
                index: None,
            },
            region: body,
            host: None,
            span: sp(),
        },
    ))
}

fn lowered<'a>(a: &'a Allocator, root: Region<'a>) -> Lowered<'a> {
    Lowered {
        allocator: a,
        artifact: L2TemplateParts::new("", root),
        op_count: 0,
        diagnostics: StdVec::new(),
        consumed_expressions: StdVec::new(),
        caps: LegacyCaps::VUE3,
    }
}

fn comp<'a>(
    a: &'a Allocator,
    name: &'a str,
    bindings: ArenaVec<'a, BindingOp<'a>>,
    children: Region<'a>,
) -> Op<'a> {
    Op::Component(ab(
        a,
        ComponentOp {
            name,
            attributes: av(a),
            bindings,
            children,
            span: sp(),
        },
    ))
}

fn el<'a>(
    a: &'a Allocator,
    tag: &'a str,
    bindings: ArenaVec<'a, BindingOp<'a>>,
    children: Region<'a>,
) -> Op<'a> {
    Op::Element(ab(
        a,
        ElementOp {
            tag,
            namespace: Namespace::Html,
            attributes: ArenaVec::<Attribute<'a>>::new_in(&a),
            bindings,
            children,
            span: sp(),
        },
    ))
}

fn text<'a>(a: &'a Allocator, content: &'a str) -> Op<'a> {
    Op::Text(ab(
        a,
        TextOp {
            content,
            span: sp(),
        },
    ))
}

fn compound<'a>(a: &'a Allocator) -> Op<'a> {
    Op::Interpolation(ab(
        a,
        InterpolationOp {
            expression: opaque(a, "hello {{ name }}", OpaqueReason::Compound),
            raw_html: false,
            span: sp(),
        },
    ))
}

fn js<'a>(a: &'a Allocator, source: &'a str) -> ExprRef<'a> {
    let expr = ExprRef::parse_js_in(a, source, Span::new(0, source.len() as u32));
    assert!(matches!(expr, ExprRef::Js(_)), "{source:?} must parse");
    expr
}

fn foreign<'a>(a: &'a Allocator, source: &'a str) -> ExprRef<'a> {
    ExprRef::Foreign(a.alloc(ForeignExpr {
        dialect: "test",
        source,
        span: sp(),
        facts: av(a),
    }))
}

fn opaque<'a>(a: &'a Allocator, source: &'a str, reason: OpaqueReason) -> ExprRef<'a> {
    ExprRef::Opaque(a.alloc(OpaqueExpr {
        reason,
        source,
        span: sp(),
    }))
}

fn region<'a, const N: usize>(a: &'a Allocator, ops: [Op<'a>; N]) -> Region<'a> {
    Region {
        ops: ArenaVec::from_iter_in(ops, &a),
    }
}

fn empty_region<'a>(a: &'a Allocator) -> Region<'a> {
    Region { ops: av(a) }
}

fn av<'a, T: 'a>(a: &'a Allocator) -> ArenaVec<'a, T> {
    ArenaVec::new_in(&a)
}

fn av1<'a, T: 'a>(a: &'a Allocator, value: T) -> ArenaVec<'a, T> {
    ArenaVec::from_iter_in([value], &a)
}

fn ab<'a, T: 'a>(a: &'a Allocator, value: T) -> ArenaBox<'a, T> {
    ArenaBox::new_in(value, &a)
}

fn id(index: u32) -> NodeId {
    NodeId::from_index(index).expect("test id is below the node limit")
}

fn sp() -> Span {
    Span::new(0, 1)
}

#[test]
fn composed_analysis_visits_the_host_empty_arm_at_its_enclosing_depth() {
    use vize_l0::pass::NoRemarks;
    use vize_l2::analysis::{
        static_facts::StaticLevel, template, template_complexity::DecisionKind,
    };
    use vize_l2::provenance::NoProvenance;

    let arena = Allocator::new();
    let (mut lowered, _) = host_for_loop(&arena);
    let Some(Op::For(iteration)) = lowered.artifact.root.ops.first_mut() else {
        panic!("host loop fixture");
    };
    let empty_owner = el(&arena, "b", av(&arena), empty_region(&arena));
    let inner = for_op(&arena, js(&arena, "item"), region(&arena, [empty_owner]));
    iteration.host.as_mut().expect("host fixture").empty = Some(region(&arena, [inner]));
    let result = template::run(&lowered.artifact.root, 4, &mut NoProvenance, &mut NoRemarks);
    assert_eq!(result.op_visits, 4);
    assert_eq!(result.complexity.max_nesting, 1);
    assert_eq!(result.complexity.cognitive, 2);
    let rows: StdVec<_> = result
        .complexity
        .contributions
        .iter()
        .map(|row| (row.kind, row.op, row.nesting))
        .collect();
    assert_eq!(
        rows,
        [
            (DecisionKind::For, NodeId::from_index(0), 0),
            (DecisionKind::For, NodeId::from_index(2), 0),
        ]
    );
    let owners = result.static_facts.sorted_entries();
    assert_eq!(owners.len(), 1);
    assert_eq!(owners[0].0.index(), 3);
    assert_eq!(owners[0].1.level, StaticLevel::FullyStatic);
}
