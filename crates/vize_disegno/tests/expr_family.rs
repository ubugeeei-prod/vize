//! The exhaustive-match canary for the S2 expression family (P2-5b;
//! Vue 2 filters P2-9). Split from `op_family.rs` under the source
//! budget when `vue.filter` joined the closed set.

use vize_disegno::expr::{ExprRef, ForeignExpr, JsExpr, OpaqueExpr, OpaqueReason, VueFilterExpr};
use vize_s0::{Allocator, Span, Vec};

/// No `_` arm: a new `ExprRef` variant must break this match.
fn expr_keyword(expr: &ExprRef<'_>) -> &'static str {
    match expr {
        ExprRef::Js(_) => "js",
        ExprRef::Foreign(_) => "foreign",
        ExprRef::Filter(_) => "vue.filter",
        ExprRef::Opaque(_) => "opaque",
    }
}

/// No `_` arm: a new escape class must break this match.
fn reason_keyword(reason: OpaqueReason) -> &'static str {
    match reason {
        OpaqueReason::ForValue => "for-value",
        OpaqueReason::MultiStatement => "multi-statement",
        OpaqueReason::NestingRefused => "nesting-refused",
        OpaqueReason::ParseRejected => "parse-rejected",
        OpaqueReason::Compound => "compound",
    }
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
        ExprRef::Filter(
            VueFilterExpr::parse_in(allocator, "a + b | id", Span::new(0, 10))
                .expect("a filter chain is admitted"),
        ),
        ExprRef::Opaque(allocator.alloc(OpaqueExpr {
            reason: OpaqueReason::Compound,
            source: "a + b",
            span: Span::new(0, 5),
        })),
    ];
    let keywords: std::vec::Vec<&str> = exprs.iter().map(expr_keyword).collect();
    assert_eq!(keywords, ["js", "foreign", "vue.filter", "opaque"]);
    let sources: std::vec::Vec<&str> = exprs.iter().map(|expr| expr.source()).collect();
    assert_eq!(sources, ["a + b", "a + b", "a + b | id", "a + b"]);
    for expr in &exprs {
        assert_eq!(expr.mnemonic(), expr_keyword(expr));
    }
    let reasons = [
        OpaqueReason::ForValue,
        OpaqueReason::MultiStatement,
        OpaqueReason::NestingRefused,
        OpaqueReason::ParseRejected,
        OpaqueReason::Compound,
    ];
    for reason in reasons {
        assert_eq!(reason_keyword(reason), reason.mnemonic());
    }
}
