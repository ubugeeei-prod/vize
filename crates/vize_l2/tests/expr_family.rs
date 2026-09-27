//! The exhaustive-match canary for the L2 expression family (P2-5b;
//! Vue 2 filters P2-9). Split from `op_family.rs` under the source
//! budget when `vue.filter` joined the closed set.

use oxc_span::GetSpan;
use vize_l0::{Allocator, Span, Vec};
use vize_l2::expr::{ExprRef, ForeignExpr, JsExpr, OpaqueExpr, OpaqueReason, VueFilterExpr};

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

#[test]
fn js_expression_admits_trailing_block_comment_trivia_only() {
    let arena = Allocator::default();
    let allocator = &arena;
    let with_block_comment = "i % 3 === 0 /* perf optimization */";
    let js = JsExpr::parse_in(
        allocator,
        with_block_comment,
        Span::new(0, with_block_comment.len() as u32),
    )
    .expect("closed trailing block comments are trivia");
    assert_eq!(js.source, with_block_comment);

    let with_statement_tail = "i % 3 === 0; /* perf optimization */";
    assert_eq!(
        JsExpr::parse_in(
            allocator,
            with_statement_tail,
            Span::new(0, with_statement_tail.len() as u32),
        )
        .unwrap_err(),
        OpaqueReason::ParseRejected
    );

    let with_line_comment = "i % 3 === 0 // perf optimization";
    assert_eq!(
        JsExpr::parse_in(
            allocator,
            with_line_comment,
            Span::new(0, with_line_comment.len() as u32),
        )
        .unwrap_err(),
        OpaqueReason::ParseRejected
    );
}

#[test]
fn word_safety_keeps_oxc_admission_and_authored_payloads() {
    let allocator = Allocator::default();
    for source in [
        "item", "_", "$", "$event", "x1", "true", "null", "値", r"\u0061",
    ] {
        let span = Span::new(7, 7 + source.len() as u32);
        let js = JsExpr::parse_in(&allocator, source, span).expect("word expression is admitted");
        assert_eq!(js.source, source);
        assert_eq!(js.span, span);
        assert_eq!(js.ast.span(), oxc_span::Span::new(0, source.len() as u32));
        match (source, js.ast) {
            ("true", oxc_ast::ast::Expression::BooleanLiteral(literal)) => {
                assert!(literal.value);
            }
            ("null", oxc_ast::ast::Expression::NullLiteral(_)) => {}
            (name, oxc_ast::ast::Expression::Identifier(identifier))
                if name != "true" && name != "null" =>
            {
                let expected = if name == r"\u0061" { "a" } else { name };
                assert_eq!(identifier.name.as_str(), expected);
            }
            _ => panic!("unexpected OXC AST for {source}: {:?}", js.ast),
        }
    }
    for source in [
        "",
        "for",
        "return",
        "class",
        "typeof",
        "item key",
        "item; item",
    ] {
        assert_eq!(
            JsExpr::parse_in(&allocator, source, Span::new(0, source.len() as u32)).unwrap_err(),
            OpaqueReason::ParseRejected,
        );
    }
}

#[test]
fn guarded_word_suffixes_retain_nesting_refusal_before_oxc() {
    let allocator = Allocator::default();
    let depth = vize_l0::expression_guard::MAX_EXPRESSION_NESTING_DEPTH + 1;
    for source in [
        format!("word{}x{}", "(".repeat(depth), ")".repeat(depth)),
        format!("{}x", "!".repeat(depth)),
        format!("{}x", "typeof ".repeat(depth)),
        "1".repeat(4097),
    ] {
        assert_eq!(
            JsExpr::parse_in(&allocator, &source, Span::new(7, 7 + source.len() as u32))
                .unwrap_err(),
            OpaqueReason::NestingRefused,
        );
    }
}
