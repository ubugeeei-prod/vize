//! Replay of the `js_ts_expression` slow unit from #6401.
//!
//! The 474-byte input has 43 speculative type-angle starts interspersed with
//! `>` closers. Its peak angle depth stays under the parser's recursion limit,
//! but OXC retries the nested failed type-argument branches for 11 seconds in
//! the fuzz job (23 seconds in a local debug build). The guard must reject it
//! before any production expression parser or rewrite reaches OXC.

use vize_atelier_core::steps::expression::{
    expression_has_balanced_delimiters, expression_is_safe_to_parse, expression_nesting_depth,
    prefix_identifiers_in_expression, strip_typescript_from_expression,
};
use vize_atelier_core::steps::v_slot::extract_slot_prop_names;

/// Exact `slow-unit-fb961363155ca85c6145c402247d032b2fbc04f6` bytes from
/// the `fuzz-reproducers-js_ts_expression` artifact of run 35842448250.
const REPRODUCER: &str = include_str!("fixtures/js_ts_expression_slow_unit_6401.txt");

#[test]
fn issue_6401_slow_unit_is_rejected_before_parsing() {
    assert_eq!(REPRODUCER.len(), 474);
    assert_eq!(REPRODUCER.matches('<').count(), 43);
    assert_eq!(REPRODUCER.matches('>').count(), 31);
    assert_eq!(expression_nesting_depth(REPRODUCER), 30);
    assert!(expression_has_balanced_delimiters(REPRODUCER));

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
fn independent_generic_calls_do_not_exhaust_the_speculation_budget() {
    // Each closed generic call has its own parser branch. A source containing
    // many such calls must not inherit the count from earlier branches.
    let source = format!("{}f<T>(x)", "f<T>(x) + ".repeat(64));
    assert!(expression_has_balanced_delimiters(&source));
    assert!(expression_is_safe_to_parse(&source));
}
