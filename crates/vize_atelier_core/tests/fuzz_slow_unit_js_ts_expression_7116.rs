//! Replay of the `js_ts_expression` OOM reproducer from #7116.
//!
//! The 1825-byte input nests six `<(ident =` arrow-parameter defaults, and
//! each frame holds a run of `<` comparisons joined by `||`. OXC speculates
//! that shape as a function type inside a type-argument list; `||` in the
//! default does not end the speculation, and the reparse multiplies until the
//! process runs out of memory. The guard must reject the bytes before any
//! expression parser or rewrite calls OXC. This test does not parse them.
#![expect(clippy::disallowed_macros, reason = "insta and fixtures use format!")]

use vize_atelier_core::steps::expression::{
    expression_has_balanced_delimiters, expression_is_safe_to_parse, expression_nesting_depth,
    prefix_identifiers_in_expression, strip_typescript_from_expression,
};
use vize_atelier_core::steps::v_slot::extract_slot_prop_names;

/// Exact `oom-57f8ceea7865e02bc9246c6459486b7bada6c09e` bytes from the
/// `fuzz-reproducers-js_ts_expression` artifact of run 36413054065.
const REPRODUCER: &str = include_str!("fixtures/js_ts_expression_oom_7116.bin");

#[test]
fn issue_7116_oom_reproducer_is_rejected_before_parsing() {
    assert_eq!(REPRODUCER.len(), 1825);
    assert_eq!(REPRODUCER.matches('<').count(), 66);
    assert_eq!(REPRODUCER.matches('>').count(), 0);
    assert_eq!(REPRODUCER.matches("||").count(), 142);
    assert!(expression_has_balanced_delimiters(REPRODUCER));
    assert!(expression_nesting_depth(REPRODUCER) <= 31);

    assert!(!expression_is_safe_to_parse(REPRODUCER));
    assert_eq!(
        prefix_identifiers_in_expression(REPRODUCER).as_str(),
        REPRODUCER
    );
    assert_eq!(
        strip_typescript_from_expression(REPRODUCER).as_str(),
        REPRODUCER
    );
    assert_eq!(format!("{:?}", extract_slot_prop_names(REPRODUCER)), "[]");
}

#[test]
fn nested_arrow_defaults_with_one_comparison_per_frame_stay_safe() {
    // Each outer frame holds a single `<` (`aN<(bN=`), so the product stays
    // linear. Depth 7 of plain `<(ident =` parsed in under a millisecond.
    let mut pure = String::new();
    for index in 0..7 {
        pure.push('a');
        pure.push_str(&index.to_string());
        pure.push_str("<(b");
        pure.push_str(&index.to_string());
        pure.push('=');
    }
    pure.push('1');
    pure.push_str(&")".repeat(7));
    assert!(expression_is_safe_to_parse(&pure), "{pure}");
    assert!(
        prefix_identifiers_in_expression(&pure).contains("_ctx.a0"),
        "{pure}"
    );
}

#[test]
fn flat_comparison_tail_inside_two_arrow_defaults_stays_safe() {
    // The inner frame holds the whole tail and the outer frame holds one `<`.
    // That is one branching frame, which OXC walks linearly.
    let mut flat = String::from("a<(b=c<(d=");
    flat.push_str(&"x<y||".repeat(39));
    flat.push_str("x<y))");
    assert!(expression_is_safe_to_parse(&flat), "{flat}");
    let rewritten = prefix_identifiers_in_expression(&flat);
    assert_ne!(rewritten.as_str(), flat);
    assert!(rewritten.starts_with("_ctx.a<"), "{rewritten}");
}

#[test]
fn arrow_default_frames_that_each_hold_a_comparison_run_are_rejected() {
    // Four frames of eight `<` comparisons: the same family as #7116, short
    // enough to spell out, with a product past 8 × 31².
    let heavy = nested_arrow_defaults(4, 8);
    assert!(!expression_is_safe_to_parse(&heavy));
    assert_eq!(prefix_identifiers_in_expression(&heavy).as_str(), heavy);

    let light = nested_arrow_defaults(4, 1);
    assert!(expression_is_safe_to_parse(&light));
}

#[test]
fn ordinary_arrow_defaults_and_parenthesized_comparisons_stay_safe() {
    assert!(expression_is_safe_to_parse(
        "useHandler<(payload: MouseEvent) => void>(handler)"
    ));
    assert!(expression_is_safe_to_parse("f<(A)>(x) + g<(B)>(y)"));
    assert!(expression_is_safe_to_parse("a<(b = 1) && c<(d = 2)"));

    let mut chain = String::new();
    for index in 0..40 {
        if index > 0 {
            chain.push_str(" && ");
        }
        chain.push_str("a < (b + 1)");
    }
    assert!(expression_is_safe_to_parse(&chain));
    assert!(prefix_identifiers_in_expression(&chain).contains("_ctx.a < (_ctx.b + 1)"));
}

fn nested_arrow_defaults(levels: usize, chains: usize) -> String {
    let mut body = String::new();
    for _ in 0..levels {
        body.push_str("a<b<c<\n age<(stz=k||=");
        body.push_str(&"x<y||".repeat(chains));
    }
    body.push_str(&")".repeat(levels));
    body
}
