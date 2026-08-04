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
fn expression_guard_ends_unterminated_strings_at_line_terminators() {
    // The 4 KiB js_ts_expression reproducer (#3213) hid its pathological
    // bracket and type-angle runs behind an unterminated `'` literal: the
    // scanner ran through newlines to a quote hundreds of bytes later while
    // the lexer ends a (mal)formed string literal at the first unescaped LF or
    // CR and resumes lexing the junk the guard never saw.
    for terminator in ["\n", "\r"] {
        let hidden = [
            "'x",
            terminator,
            &"(".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1),
        ]
        .concat();
        assert_eq!(
            expression_nesting_depth(&hidden),
            MAX_EXPRESSION_NESTING_DEPTH + 1,
            "terminator {:?}",
            terminator.escape_unicode()
        );
        assert!(expression_exceeds_max_depth(&hidden));
        assert!(!expression_is_safe_to_parse(&hidden));
    }

    // Unescaped LS/PS are legal string content (ES2019), and `\` + LF as well
    // as `\` + CRLF are LineContinuation sequences: none of them end the
    // literal, so none of them expose the quoted brackets to the guard.
    for continued in [
        "'a\u{2028}b(' + c",
        "'a\u{2029}b(' + c",
        "'a\\\nb(' + c",
        "'a\\\r\nb(' + c",
    ] {
        assert_eq!(expression_nesting_depth(continued), 0, "{continued:?}");
        assert!(expression_has_balanced_delimiters(continued));
        assert!(expression_is_safe_to_parse(continued));
    }
}

#[test]
fn expression_guard_ends_line_comments_at_every_line_terminator() {
    // The js_ts_expression OOM reproducer (#3185) hid parser-visible source
    // behind a bare CR inside a line comment: the scanner only ended comments
    // at LF while the lexer ends them at any ECMAScript line terminator, so
    // everything between the CR and the next LF escaped the depth guard.
    for terminator in ["\r", "\u{2028}", "\u{2029}"] {
        let hidden = [
            "//x",
            terminator,
            &"(".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1),
        ]
        .concat();
        assert_eq!(
            expression_nesting_depth(&hidden),
            MAX_EXPRESSION_NESTING_DEPTH + 1,
            "terminator {:?}",
            terminator.escape_unicode()
        );
        assert!(expression_exceeds_max_depth(&hidden));
        assert!(!expression_has_balanced_delimiters(&hidden));
        assert!(!expression_is_safe_to_parse(&hidden));
    }

    // Without a line terminator the brackets stay inside the comment.
    let commented = ["//x ", &"(".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1)].concat();
    assert_eq!(expression_nesting_depth(&commented), 0);
    assert!(expression_is_safe_to_parse(&commented));

    // A regex literal is likewise terminated by LS/PS, not only LF/CR.
    let regex_hidden = [
        "a = /x",
        "\u{2028}",
        &"[".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1),
    ]
    .concat();
    assert!(expression_exceeds_max_depth(&regex_hidden));
    assert!(!expression_is_safe_to_parse(&regex_hidden));
}

#[test]
fn expression_guard_treats_slash_after_a_type_argument_close_as_division() {
    // Minimized from the js_ts_expression fuzz reproducer (#3858), whose
    // `…Props<{pilest c\n}>/(((…` lines repeat for 2.6 KiB. A `>` that pays back
    // an open angle closes a type-argument list, and a `/` after one is
    // division — but the guard used to allow a regex after every `>`, so
    // `skip_regex` swallowed the rest of each line and hid its bracket run from
    // the depth budget. The guard reported depth 28 on an input carrying 1394
    // unclosed brackets, so OXC saw it and recursed until the stack overflowed.
    let hidden = ["Props<{a}>/", &"(".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1)].concat();
    assert_eq!(
        expression_nesting_depth(&hidden),
        MAX_EXPRESSION_NESTING_DEPTH + 1
    );
    assert!(expression_exceeds_max_depth(&hidden));
    assert!(!expression_has_balanced_delimiters(&hidden));
    assert!(!expression_is_safe_to_parse(&hidden));
    assert_eq!(
        prefix_identifiers_in_expression(&hidden).as_str(),
        hidden.as_str()
    );

    // Only the angle-closing `>` loses its regex position. A relational `>` with
    // nothing outstanding still precedes one, so the brackets inside stay hidden.
    let relational = [
        "a > /",
        &"(".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1),
        "/.test(b)",
    ]
    .concat();
    assert_eq!(expression_nesting_depth(&relational), 1);
    assert!(!expression_exceeds_max_depth(&relational));
    assert!(expression_is_safe_to_parse(&relational));
}

#[test]
fn expression_guard_ends_regex_at_line_terminator_after_backslash() {
    // A regex literal cannot span a line terminator, and `\` immediately before
    // one is not a valid escape: the lexer ends the regex there. The guard's
    // 2-byte escape skip used to swallow the terminator (LF/CR) or its 0xE2 lead
    // byte (LS/PS), hiding the trailing brackets from the depth budget.
    for terminator in ["\n", "\r", "\u{2028}", "\u{2029}"] {
        let hidden = [
            "a = /x\\",
            terminator,
            &"[".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1),
        ]
        .concat();
        assert!(
            expression_exceeds_max_depth(&hidden),
            "terminator {:?}",
            terminator.escape_unicode()
        );
        assert!(
            !expression_is_safe_to_parse(&hidden),
            "terminator {:?}",
            terminator.escape_unicode()
        );
    }
}
