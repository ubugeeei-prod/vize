use vize_atelier_core::steps::expression::{
    MAX_EXPRESSION_NESTING_DEPTH, expression_exceeds_max_depth, expression_nesting_depth,
    is_event_handler_reference_expression, is_function_expression,
    prefix_identifiers_in_expression, strip_typescript_from_expression,
};

const TIMEOUT_REPRODUCER: &str = "\nd=\nd=efnuiProps<{[dnuiProps<{[d=efnuiProps<{[dnuiProps<{[defnuiProps<{[  cts<{[  oefnuiProps<{[dnuiProps<{[defnuiProps<{[  cts<{[  oo>>efnuiProps<{[  cts<{[  oefnuiProps<{[dnuiProps<{[defnuiProps<{[  cts<{[  oo>>> \x1a\x1a\x1a\x1a\x1atr";
const SLOW_TYPE_ARGUMENT_REPRODUCER: &str = "eu.tfpS<{[ erntro-Coro-Couo<tidon<cttnS<{[ erntro-rntro-Coro-Couo<tidon<cttnS<{[ erntro-Coro-Couo<tidon<cttn<tuo<tidon<cttnS<{[ erntro-Coro-Coug<tidon<cttn<tidon<cttopti<nn<ctstrCoro[Couo<tidon<cttn<tuo<tidon<cttnS<{[ erntro-Coro-Couo<tidon<cttn<tidon<cttopti<nn<ctstrin<n<<on<ta<ts";
const TIMEOUT_TYPE_ARGUMENT_REPRODUCER: &str = "eu.tfpS<{[ erntro-Coro-Couo<tidon<cttnS<{[ erntro-rntro-Coro-Couo<tidon<cttnS<{[ erntro-Coro-Couo<tidon<cttn<tuo<tidon<cttnS<{[ erntro-Coro-Couo<tidon<cttn<tidottn<tuo<tidon<cttnS<{[ erntro-Coro-Couo<tidon<cttn<tidon<cttopti<nn<ctsttro-Coro-Couo<tidon<cttnS<{[ erntro-rntro-Coro-Couo<tidon<cttnS<{[ erntro-Coro-Couo<tidon<cttn<tuo<tidon<cttnS<{[ erntro-Coro-Couo<tidon<cttn<tidottn<tuo<tidon<cttnS<{[ erntro-Coro-Couo<tidon<cttn<tidon<cttoptrin<n<<on<ta<ts";

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
