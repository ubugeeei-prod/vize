//! Replay of the `js_ts_expression` slow unit from #9931.

use vize_atelier_core::steps::expression::{
    expression_has_balanced_delimiters, expression_is_safe_to_parse, expression_nesting_depth,
    prefix_identifiers_in_expression, strip_typescript_from_expression,
};

/// Exact `slow-unit-3464555815191b27223aabc2da2fa2c0f61ea14d` bytes from the
/// `fuzz-reproducers-js_ts_expression` artifact of run 39401009352.
const REPRODUCER: &str =
    include_str!("../../../tests/fuzz/regressions/js_ts_expression/issue-9931.input");

#[test]
fn issue_9931_slow_unit_is_rejected_before_parsing() {
    assert_eq!(REPRODUCER.len(), 23_134);
    assert_eq!(REPRODUCER.matches('`').count(), 1);
    // The guard's delimiter and depth scans used to accept this malformed
    // template tail, hiding the regex body that OXC spent over a second parsing.
    assert!(expression_has_balanced_delimiters(REPRODUCER));
    assert_eq!(expression_nesting_depth(REPRODUCER), 2);
    assert!(!expression_is_safe_to_parse(REPRODUCER));
    assert_eq!(
        prefix_identifiers_in_expression(REPRODUCER).as_str(),
        REPRODUCER
    );
    assert_eq!(
        strip_typescript_from_expression(REPRODUCER).as_str(),
        REPRODUCER
    );
}

#[test]
fn closed_templates_and_interpolations_stay_safe() {
    assert!(expression_is_safe_to_parse("`plain`"));
    assert!(expression_is_safe_to_parse("`value: ${item}`"));
    assert!(!expression_is_safe_to_parse("`unterminated"));
    assert!(!expression_is_safe_to_parse("`value: ${item} tail"));
}
