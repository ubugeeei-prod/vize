//! Replay of the `js_ts_expression` long numeric-token slow units from #5733/#5737.

use vize_atelier_core::steps::expression::{
    expression_has_balanced_delimiters, expression_is_safe_to_parse, expression_nesting_depth,
    prefix_identifiers_in_expression, strip_typescript_from_expression,
};

#[test]
fn long_hex_numeric_literal_is_rejected_before_oxc_parse() {
    let source = ["0X", &"C".repeat(4095)].concat();

    assert_eq!(source.len(), 4097);
    assert_eq!(expression_nesting_depth(&source), 0);
    assert!(expression_has_balanced_delimiters(&source));
    assert!(!expression_is_safe_to_parse(&source));
    assert_eq!(prefix_identifiers_in_expression(&source).as_str(), source);
    assert_eq!(strip_typescript_from_expression(&source).as_str(), source);
}

#[test]
fn numeric_literal_budget_accepts_the_exact_boundary() {
    let source = ["0x", &"c".repeat(4094)].concat();

    assert_eq!(source.len(), 4096);
    assert_eq!(expression_nesting_depth(&source), 0);
    assert!(expression_has_balanced_delimiters(&source));
    assert!(expression_is_safe_to_parse(&source));
}
