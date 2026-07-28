use vize_atelier_core::steps::expression::{
    MAX_EXPRESSION_NESTING_DEPTH, expression_exceeds_max_depth, expression_has_balanced_delimiters,
    expression_is_safe_to_parse, expression_nesting_depth,
};

#[test]
fn expression_guard_rejects_the_backslash_quote_hidden_reproducer() {
    // The js_ts_expression timeout reproducer (#3271) hid its bracket and
    // type-angle runs behind code-position `\'`: OXC reads a `\` before a quote
    // as a stray identifier escape, not a string opener, so the brackets stay
    // live and drive exponential type-argument speculation. The scanner used to
    // open a string at that quote and let `skip_quoted` swallow the whole run,
    // so the depth budget saw nothing. This is the input's repeating unit.
    let reproducer = "<X<\\'\\u{[([[[[[[[[{[".repeat(6);
    assert!(expression_nesting_depth(&reproducer) > MAX_EXPRESSION_NESTING_DEPTH);
    assert!(expression_exceeds_max_depth(&reproducer));
    assert!(!expression_has_balanced_delimiters(&reproducer));
    assert!(!expression_is_safe_to_parse(&reproducer));
}

#[test]
fn expression_guard_counts_brackets_after_a_backslash_quote() {
    // A `\` immediately before a quote must not open a string literal: OXC
    // reads `\'`/`\"` as a broken escape, so the brackets that follow stay live
    // code and must reach the depth budget, not a phantom string literal.
    for quote in ["\\'", "\\\""] {
        let reproducer = [quote, &"(".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1)].concat();
        assert_eq!(
            expression_nesting_depth(&reproducer),
            MAX_EXPRESSION_NESTING_DEPTH + 1,
            "quote {quote:?}"
        );
        assert!(expression_exceeds_max_depth(&reproducer), "quote {quote:?}");
        assert!(!expression_is_safe_to_parse(&reproducer), "quote {quote:?}");
    }

    // A `\` before any other byte still leaves the following bracket to the
    // normal arms, so `\(` / `\[` / `\{` keep counting as OXC keeps them live.
    for lead in ["\\(", "\\[", "\\{"] {
        let reproducer = [lead, &"(".repeat(MAX_EXPRESSION_NESTING_DEPTH)].concat();
        assert_eq!(
            expression_nesting_depth(&reproducer),
            MAX_EXPRESSION_NESTING_DEPTH + 1,
            "lead {lead:?}"
        );
        assert!(!expression_is_safe_to_parse(&reproducer), "lead {lead:?}");
    }
}

#[test]
fn expression_guard_rejects_the_backslash_backtick_hidden_reproducer() {
    // The js_ts_expression stack-overflow reproducer (#3274) hid its bracket
    // runs behind a code-position `` \` ``: OXC reads a `\` before a backtick
    // as a stray identifier escape, not a template opener, so every following
    // bracket stays live and recurses in `parse_primary_expression` until the
    // stack overflows. The scanner used to open a phantom template literal at
    // that backtick and let `skip_template_text` swallow the whole run, so the
    // depth budget saw nothing. This is the input's repeating unit.
    let reproducer = "definePro\\`\\u{[([[[[[[,[[[[[[[[[[[[[[[[[[[[[[[[[[".repeat(2);
    assert!(expression_nesting_depth(&reproducer) > MAX_EXPRESSION_NESTING_DEPTH);
    assert!(expression_exceeds_max_depth(&reproducer));
    assert!(!expression_has_balanced_delimiters(&reproducer));
    assert!(!expression_is_safe_to_parse(&reproducer));
}

#[test]
fn expression_guard_counts_brackets_after_a_backslash_backtick() {
    // A `\` immediately before a backtick must not open a template literal:
    // OXC reads `` \` `` as a broken escape, so the brackets that follow stay
    // live code and must reach the depth budget, not phantom template text.
    let reproducer = ["\\`", &"(".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1)].concat();
    assert_eq!(
        expression_nesting_depth(&reproducer),
        MAX_EXPRESSION_NESTING_DEPTH + 1
    );
    assert!(expression_exceeds_max_depth(&reproducer));
    assert!(!expression_is_safe_to_parse(&reproducer));
}

#[test]
fn expression_guard_keeps_ordinary_backslash_uses_safe() {
    // Neutralizing a post-backslash quote must not touch valid expressions: a
    // `\` legally appears in code only as a unicode identifier escape, and a
    // quote not preceded by a code-position `\` still opens a real string.
    for safe in [
        "\\u0061 + b",              // unicode identifier escape
        "'a(' + '[' + \"{\"",       // brackets inside string literals
        "'a\\'b' + c",              // escaped quote *inside* a string
        "`a\\`(b` + c",             // escaped backtick *inside* a template
        "/[({]/u.test(value)",      // brackets inside a regex literal
        "`text ${ inner + '(' }!`", // template with an interpolation
    ] {
        assert!(expression_has_balanced_delimiters(safe), "{safe:?}");
        assert!(expression_is_safe_to_parse(safe), "{safe:?}");
    }
}
