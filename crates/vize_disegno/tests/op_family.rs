//! The exhaustive-match canary for the S2 op family (P2-5a; the
//! expression family P2-5b).
//!
//! Every enum of the family is matched with **no `_` arm**, so adding a
//! variant breaks this file at compile time. The canary is proved by
//! injection, not assumed (the P0-7 staleness-check pattern): the landing
//! records (`davinci-road/plan/phase-2-records/p2-5a.md`, `p2-5b.md`)
//! document the injected variants and the resulting build failures.

use vize_carton::{Allocator, Box, Span, Vec};
use vize_disegno::expr::{ExprRef, ForeignExpr, JsExpr, OpaqueExpr, OpaqueReason};
#[cfg(feature = "_legacy")]
use vize_disegno::op::VueFilterOp;
use vize_disegno::op::{
    Attribute, BindOp, BindingContract, BindingOp, ComponentOp, DynamicName, ElementOp, ForBinding,
    ForOp, IfBranch, IfOp, InterpolationOp, ModelOp, Namespace, OnOp, Op, Region, SlotContentOp,
    SlotOp, TextOp, VueDirectiveOp,
};

/// The escape payload standing in for "some expression" wherever the op
/// under test does not care which - honest by construction, since the
/// escape makes no claim an assertion could contradict.
fn placeholder<'a>(allocator: &'a Allocator) -> ExprRef<'a> {
    ExprRef::Opaque(allocator.alloc(OpaqueExpr {
        reason: OpaqueReason::ParseRejected,
        source: "%",
        span: Span::new(0, 0),
    }))
}

/// No `_` arm, by contract: a new region op must break this match.
fn op_keyword(op: &Op<'_>) -> &'static str {
    match op {
        Op::Element(_) => "ui.element",
        Op::Component(_) => "ui.component",
        Op::Text(_) => "ui.text",
        Op::Interpolation(_) => "ui.interpolation",
        Op::If(_) => "ui.if",
        Op::For(_) => "ui.for",
        Op::Slot(_) => "ui.slot",
        // `_legacy` only: the variant does not exist in the plain shape,
        // where an unguarded arm here is itself a compile error - the
        // series-7 zero-cost pin in canary form.
        #[cfg(feature = "_legacy")]
        Op::VueFilter(_) => "vue.filter",
    }
}

/// No `_` arm, by contract: a new attached op must break this match.
fn binding_keyword(op: &BindingOp<'_>) -> &'static str {
    match op {
        BindingOp::Bind(_) => "ui.bind",
        BindingOp::On(_) => "ui.on",
        BindingOp::Model(_) => "ui.model",
        BindingOp::SlotContent(_) => "ui.slot-content",
        BindingOp::VueDirective(_) => "vue.directive",
    }
}

/// No `_` arm: the payload-level enums are part of the family too.
fn namespace_keyword(namespace: Namespace) -> &'static str {
    match namespace {
        Namespace::Html => "html",
        Namespace::Svg => "svg",
        Namespace::MathMl => "mathml",
    }
}

/// No `_` arm: the payload-level enums are part of the family too.
fn name_keyword(name: &DynamicName<'_>) -> &'static str {
    match name {
        DynamicName::Static(_) => "static",
        DynamicName::Dynamic(_) => "dynamic",
    }
}

/// No `_` arm: the expression family is part of the op family too.
fn expr_keyword(expr: &ExprRef<'_>) -> &'static str {
    match expr {
        ExprRef::Js(_) => "js",
        ExprRef::Foreign(_) => "foreign",
        ExprRef::Opaque(_) => "opaque",
    }
}

/// No `_` arm: the escape classes are a closed set - a new class must
/// break this match (and the review that names its semantics).
fn reason_keyword(reason: OpaqueReason) -> &'static str {
    match reason {
        OpaqueReason::ForValue => "for-value",
        OpaqueReason::MultiStatement => "multi-statement",
        OpaqueReason::NestingRefused => "nesting-refused",
        OpaqueReason::ParseRejected => "parse-rejected",
        OpaqueReason::Compound => "compound",
        #[cfg(feature = "_legacy")]
        OpaqueReason::LegacyFilter => "legacy-filter",
    }
}

fn region<'a>(allocator: &'a Allocator) -> Region<'a> {
    Region {
        ops: Vec::new_in(&allocator),
    }
}

/// One instance of every variant, built in a real arena.
fn every_op<'a>(allocator: &'a Allocator) -> Vec<'a, Op<'a>> {
    let span = Span::new(0, 0);
    let expr = placeholder(allocator);
    Vec::from_iter_in(
        [
            Op::Element(Box::new_in(
                ElementOp {
                    tag: "div",
                    namespace: Namespace::Html,
                    attributes: Vec::new_in(&allocator),
                    bindings: Vec::new_in(&allocator),
                    children: region(allocator),
                    span,
                },
                &allocator,
            )),
            Op::Component(Box::new_in(
                ComponentOp {
                    name: "Widget",
                    attributes: Vec::new_in(&allocator),
                    bindings: Vec::new_in(&allocator),
                    children: region(allocator),
                    span,
                },
                &allocator,
            )),
            Op::Text(Box::new_in(
                TextOp {
                    content: "hi",
                    span,
                },
                &allocator,
            )),
            Op::Interpolation(Box::new_in(
                InterpolationOp {
                    expression: expr,
                    span,
                },
                &allocator,
            )),
            Op::If(Box::new_in(
                IfOp {
                    branches: Vec::from_iter_in(
                        [IfBranch {
                            condition: Some(expr),
                            region: region(allocator),
                            span,
                        }],
                        &allocator,
                    ),
                    span,
                },
                &allocator,
            )),
            Op::For(Box::new_in(
                ForOp {
                    binding: ForBinding {
                        source: expr,
                        value: expr,
                        key: None,
                        index: None,
                    },
                    region: region(allocator),
                    span,
                },
                &allocator,
            )),
            Op::Slot(Box::new_in(
                SlotOp {
                    name: DynamicName::Static("default"),
                    attributes: Vec::new_in(&allocator),
                    bindings: Vec::new_in(&allocator),
                    fallback: region(allocator),
                    span,
                },
                &allocator,
            )),
            #[cfg(feature = "_legacy")]
            Op::VueFilter(Box::new_in(
                VueFilterOp {
                    expression: expr,
                    span,
                },
                &allocator,
            )),
        ],
        &allocator,
    )
}

fn every_binding<'a>(allocator: &'a Allocator) -> Vec<'a, BindingOp<'a>> {
    let span = Span::new(0, 0);
    let expr = placeholder(allocator);
    Vec::from_iter_in(
        [
            BindingOp::Bind(Box::new_in(
                BindOp {
                    name: Some(DynamicName::Dynamic(expr)),
                    modifiers: Vec::from_iter_in(["camel"], &allocator),
                    value: Some(expr),
                    span,
                },
                &allocator,
            )),
            BindingOp::On(Box::new_in(
                OnOp {
                    name: None,
                    modifiers: Vec::new_in(&allocator),
                    handler: Some(expr),
                    span,
                },
                &allocator,
            )),
            BindingOp::Model(Box::new_in(
                ModelOp {
                    contract: BindingContract {
                        read: expr,
                        write: expr,
                    },
                    attributes: Vec::from_iter_in(
                        [Attribute {
                            name: "element-kind",
                            value: Some("input"),
                            span,
                        }],
                        &allocator,
                    ),
                    span,
                },
                &allocator,
            )),
            BindingOp::SlotContent(Box::new_in(
                SlotContentOp {
                    name: Some(DynamicName::Static("header")),
                    modifiers: Vec::new_in(&allocator),
                    params: Some(expr),
                    span,
                },
                &allocator,
            )),
            BindingOp::VueDirective(Box::new_in(
                VueDirectiveOp {
                    name: "pin",
                    argument: Some(DynamicName::Dynamic(expr)),
                    modifiers: Vec::from_iter_in(["lazy"], &allocator),
                    value: Some(expr),
                    span,
                },
                &allocator,
            )),
        ],
        &allocator,
    )
}

#[test]
fn every_region_op_variant_is_matched_without_a_wildcard() {
    let allocator = Allocator::default();
    let ops = every_op(&allocator);
    let keywords: std::vec::Vec<&str> = ops.iter().map(op_keyword).collect();
    #[cfg(not(feature = "_legacy"))]
    let expected: &[&str] = &[
        "ui.element",
        "ui.component",
        "ui.text",
        "ui.interpolation",
        "ui.if",
        "ui.for",
        "ui.slot",
    ];
    #[cfg(feature = "_legacy")]
    let expected: &[&str] = &[
        "ui.element",
        "ui.component",
        "ui.text",
        "ui.interpolation",
        "ui.if",
        "ui.for",
        "ui.slot",
        "vue.filter",
    ];
    assert_eq!(keywords, expected);
    for op in &ops {
        assert_eq!(op.mnemonic(), op_keyword(op));
    }
}

#[test]
fn every_attached_op_variant_is_matched_without_a_wildcard() {
    let allocator = Allocator::default();
    let bindings = every_binding(&allocator);
    let keywords: std::vec::Vec<&str> = bindings.iter().map(binding_keyword).collect();
    assert_eq!(
        keywords,
        [
            "ui.bind",
            "ui.on",
            "ui.model",
            "ui.slot-content",
            "vue.directive"
        ]
    );
    for binding in &bindings {
        assert_eq!(binding.mnemonic(), binding_keyword(binding));
    }
}

#[test]
fn every_payload_enum_variant_is_matched_without_a_wildcard() {
    let allocator = Allocator::default();
    assert_eq!(namespace_keyword(Namespace::Html), "html");
    assert_eq!(namespace_keyword(Namespace::Svg), "svg");
    assert_eq!(namespace_keyword(Namespace::MathMl), "mathml");
    assert_eq!(name_keyword(&DynamicName::Static("header")), "static");
    assert_eq!(
        name_keyword(&DynamicName::Dynamic(placeholder(&allocator))),
        "dynamic"
    );
}

#[test]
fn every_expression_variant_is_matched_without_a_wildcard() {
    let arena = Allocator::default();
    let allocator = &arena;
    let js = JsExpr::parse_in(allocator, "a + b", Span::new(0, 5)).expect("`a + b` is admitted");
    let exprs = [
        ExprRef::Js(js),
        ExprRef::Foreign(allocator.alloc(ForeignExpr {
            dialect: "moonbit",
            source: "a + b",
            span: Span::new(0, 5),
            facts: Vec::new_in(&allocator),
        })),
        ExprRef::Opaque(allocator.alloc(OpaqueExpr {
            reason: OpaqueReason::Compound,
            source: "a + b",
            span: Span::new(0, 5),
        })),
    ];
    let keywords: std::vec::Vec<&str> = exprs.iter().map(expr_keyword).collect();
    assert_eq!(keywords, ["js", "foreign", "opaque"]);
    for expr in &exprs {
        assert_eq!(expr.mnemonic(), expr_keyword(expr));
        assert_eq!(expr.source(), "a + b");
        assert_eq!(expr.span(), Span::new(0, 5));
    }
    let reasons = [
        OpaqueReason::ForValue,
        OpaqueReason::MultiStatement,
        OpaqueReason::NestingRefused,
        OpaqueReason::ParseRejected,
        OpaqueReason::Compound,
        #[cfg(feature = "_legacy")]
        OpaqueReason::LegacyFilter,
    ];
    for reason in reasons {
        assert_eq!(reason_keyword(reason), reason.mnemonic());
    }
}
