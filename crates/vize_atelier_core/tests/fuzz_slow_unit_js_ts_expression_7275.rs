use vize_atelier_core::steps::expression::expression_is_safe_to_parse;

#[test]
fn postfix_non_null_assertion_does_not_hide_a_deep_paren_run_as_regex() {
    // `s!` is a TypeScript postfix assertion. The following `/=` is division
    // assignment, so the parentheses remain visible to OXC's recursive parser.
    let source = format!("s!/={}{}", "(".repeat(64), "/=tail");
    assert!(!expression_is_safe_to_parse(&source));
    assert!(!expression_is_safe_to_parse(include_str!(
        "fixtures/js_ts_expression_7275.txt"
    )));
    assert!(expression_is_safe_to_parse("! /foo/.test(value)"));
}
