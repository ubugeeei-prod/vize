use vize_atelier_core::steps::expression::{
    MAX_EXPRESSION_NESTING_DEPTH, expression_exceeds_max_depth, expression_has_balanced_delimiters,
    expression_is_safe_to_parse, expression_nesting_depth, is_event_handler_reference_expression,
    is_function_expression, prefix_identifiers_in_expression, strip_typescript_from_expression,
};

const TIMEOUT_REPRODUCER: &str = "\nd=\nd=efnuiProps<{[dnuiProps<{[d=efnuiProps<{[dnuiProps<{[defnuiProps<{[  cts<{[  oefnuiProps<{[dnuiProps<{[defnuiProps<{[  cts<{[  oo>>efnuiProps<{[  cts<{[  oefnuiProps<{[dnuiProps<{[defnuiProps<{[  cts<{[  oo>>> \x1a\x1a\x1a\x1a\x1atr";
const SLOW_TYPE_ARGUMENT_REPRODUCER: &str = "eu.tfpS<{[ erntro-Coro-Couo<tidon<cttnS<{[ erntro-rntro-Coro-Couo<tidon<cttnS<{[ erntro-Coro-Couo<tidon<cttn<tuo<tidon<cttnS<{[ erntro-Coro-Coug<tidon<cttn<tidon<cttopti<nn<ctstrCoro[Couo<tidon<cttn<tuo<tidon<cttnS<{[ erntro-Coro-Couo<tidon<cttn<tidon<cttopti<nn<ctstrin<n<<on<ta<ts";
const TIMEOUT_TYPE_ARGUMENT_REPRODUCER: &str = "eu.tfpS<{[ erntro-Coro-Couo<tidon<cttnS<{[ erntro-rntro-Coro-Couo<tidon<cttnS<{[ erntro-Coro-Couo<tidon<cttn<tuo<tidon<cttnS<{[ erntro-Coro-Couo<tidon<cttn<tidottn<tuo<tidon<cttnS<{[ erntro-Coro-Couo<tidon<cttn<tidon<cttopti<nn<ctsttro-Coro-Couo<tidon<cttnS<{[ erntro-rntro-Coro-Couo<tidon<cttnS<{[ erntro-Coro-Couo<tidon<cttn<tuo<tidon<cttnS<{[ erntro-Coro-Couo<tidon<cttn<tidottn<tuo<tidon<cttnS<{[ erntro-Coro-Couo<tidon<cttn<tidon<cttoptrin<n<<on<ta<ts";
const MISMATCHED_DELIMITER_REPRODUCER: &str = "\ndennProps<[\n  ao?: stri,\t\n);\n";

#[test]
fn expression_guard_rejects_the_js_ts_timeout_reproducer() {
    assert_eq!(TIMEOUT_REPRODUCER.len(), 222);
    assert_eq!(expression_nesting_depth(TIMEOUT_REPRODUCER), 46);
    assert!(expression_exceeds_max_depth(TIMEOUT_REPRODUCER));
    assert!(!is_event_handler_reference_expression(TIMEOUT_REPRODUCER));
    assert!(!is_function_expression(TIMEOUT_REPRODUCER));
    assert_eq!(
        prefix_identifiers_in_expression(TIMEOUT_REPRODUCER).as_str(),
        TIMEOUT_REPRODUCER
    );
    assert_eq!(
        strip_typescript_from_expression(TIMEOUT_REPRODUCER).as_str(),
        TIMEOUT_REPRODUCER
    );
}

#[test]
fn expression_guard_preserves_the_documented_boundary() {
    let allowed = "(".repeat(MAX_EXPRESSION_NESTING_DEPTH);
    let rejected = "(".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1);

    assert!(!expression_exceeds_max_depth(&allowed));
    assert!(expression_exceeds_max_depth(&rejected));
}

#[test]
fn expression_guard_rejects_the_mismatched_delimiter_reproducer() {
    assert_eq!(MISMATCHED_DELIMITER_REPRODUCER.len(), 30);
    assert_eq!(expression_nesting_depth(MISMATCHED_DELIMITER_REPRODUCER), 1);
    assert!(!expression_has_balanced_delimiters(
        MISMATCHED_DELIMITER_REPRODUCER
    ));
    assert!(!expression_is_safe_to_parse(
        MISMATCHED_DELIMITER_REPRODUCER
    ));
    assert!(!is_event_handler_reference_expression(
        MISMATCHED_DELIMITER_REPRODUCER
    ));
    assert!(!is_function_expression(MISMATCHED_DELIMITER_REPRODUCER));
    assert_eq!(
        prefix_identifiers_in_expression(MISMATCHED_DELIMITER_REPRODUCER).as_str(),
        MISMATCHED_DELIMITER_REPRODUCER
    );
    assert_eq!(
        strip_typescript_from_expression(MISMATCHED_DELIMITER_REPRODUCER).as_str(),
        MISMATCHED_DELIMITER_REPRODUCER
    );
}

#[test]
fn expression_guard_tracks_delimiter_kinds_without_scanning_literals() {
    let balanced = r#"foo("]") + /[})]/u.test(value) + `] ${items.map((item) => ({ item }))}`"#;
    assert!(expression_has_balanced_delimiters(balanced));
    assert!(expression_is_safe_to_parse(balanced));

    for mismatched in ["(]", "([)]", "foo(", "foo)", "`text ${foo)`"] {
        assert!(!expression_has_balanced_delimiters(mismatched));
        assert!(!expression_is_safe_to_parse(mismatched));
    }
}

#[test]
fn expression_guard_rejects_decorator_chain_reproducer() {
    let rejected = "@".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1) + "value";
    let allowed = "@".repeat(MAX_EXPRESSION_NESTING_DEPTH) + "value";
    assert!(expression_exceeds_max_depth(&rejected));
    assert!(!expression_exceeds_max_depth(&allowed));
    assert_eq!(
        prefix_identifiers_in_expression(&rejected).as_str(),
        rejected
    );
    assert_eq!(expression_nesting_depth(r#""user@example.com""#), 0);
    assert_eq!(
        expression_nesting_depth("value /* @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@ */"),
        0
    );
}

#[test]
fn expression_guard_ignores_at_signs_inside_regex_literals() {
    let at_signs = "@".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1);
    let expression = ["/[", at_signs.as_str(), "]+/gu.test(value)"].concat();
    let expected = expression.replace("value", "_ctx.value");
    assert_eq!(expression_nesting_depth(&expression), 1);
    assert!(!expression_exceeds_max_depth(&expression));
    assert_eq!(
        prefix_identifiers_in_expression(&expression).as_str(),
        expected
    );
}

#[test]
fn expression_guard_rejects_nested_type_argument_reproducers() {
    assert_eq!(SLOW_TYPE_ARGUMENT_REPRODUCER.len(), 282);
    assert_eq!(TIMEOUT_TYPE_ARGUMENT_REPRODUCER.len(), 456);

    for reproducer in [
        SLOW_TYPE_ARGUMENT_REPRODUCER,
        TIMEOUT_TYPE_ARGUMENT_REPRODUCER,
    ] {
        assert!(expression_nesting_depth(reproducer) > MAX_EXPRESSION_NESTING_DEPTH);
        assert!(expression_exceeds_max_depth(reproducer));
        assert!(!is_event_handler_reference_expression(reproducer));
        assert!(!is_function_expression(reproducer));
        assert_eq!(
            prefix_identifiers_in_expression(reproducer).as_str(),
            reproducer
        );
        assert_eq!(
            strip_typescript_from_expression(reproducer).as_str(),
            reproducer
        );
    }
}

#[test]
fn expression_guard_scans_deep_template_interpolations_without_recursion() {
    let depth = 100_000;
    let mut expression = "(".repeat(depth);
    expression.insert_str(0, "`literal ${");
    expression.push_str("value");
    expression.push_str(&")".repeat(depth));
    expression.push_str("}`");

    assert!(expression_exceeds_max_depth(&expression));
    assert_eq!(
        prefix_identifiers_in_expression(&expression).as_str(),
        expression
    );
}

#[test]
fn expression_guard_counts_brackets_after_a_backslash_slash_sequence() {
    // Minimized from the js_ts_expression fuzz crash (#3107): after `v`, the
    // `\` byte must not enable regex-literal detection, otherwise the
    // following `/` swallows the rest of the source as a "regex" and the
    // bracket run is hidden from the depth guard while OXC still recurses
    // over every real `(`.
    let reproducer = ["v\\/", &"(".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1)].concat();
    assert_eq!(reproducer.len(), 35);
    assert_eq!(
        expression_nesting_depth(&reproducer),
        MAX_EXPRESSION_NESTING_DEPTH + 1
    );
    assert!(expression_exceeds_max_depth(&reproducer));
    assert!(!expression_has_balanced_delimiters(&reproducer));
    assert!(!expression_is_safe_to_parse(&reproducer));
    assert!(!is_event_handler_reference_expression(&reproducer));
    assert!(!is_function_expression(&reproducer));
    assert_eq!(
        prefix_identifiers_in_expression(&reproducer).as_str(),
        reproducer
    );
    assert_eq!(
        strip_typescript_from_expression(&reproducer).as_str(),
        reproducer
    );

    // The stack-overflow shape found by the fuzzer: tens of thousands of
    // brackets hidden behind the same two-byte prefix.
    let overflow = ["v\\/", &"(".repeat(60_000)].concat();
    assert_eq!(expression_nesting_depth(&overflow), 60_000);
    assert!(!expression_is_safe_to_parse(&overflow));
}

#[test]
fn expression_guard_treats_slash_after_private_names_as_division() {
    let expression = "a.#b / c / d";
    assert_eq!(expression_nesting_depth(expression), 0);
    assert!(expression_has_balanced_delimiters(expression));
    assert!(expression_is_safe_to_parse(expression));
}

#[test]
fn expression_guard_does_not_treat_relational_operators_as_type_angles() {
    let expression = std::iter::repeat_n("value < limit", MAX_EXPRESSION_NESTING_DEPTH + 1)
        .collect::<Vec<_>>()
        .join(" || ");

    assert_eq!(expression_nesting_depth(&expression), 0);
    assert!(!expression_exceeds_max_depth(&expression));
    let rewritten = prefix_identifiers_in_expression(&expression);
    assert!(rewritten.contains("_ctx.value < _ctx.limit"), "{rewritten}");
}
