//! Regex-literal boundaries in the expression nesting guard.
//!
//! The byte scanner has to stop exactly where OXC's lexer stops. Skipping more
//! than the lexer does hides brackets and type angles from the depth budget
//! while OXC still recurses over them.

use vize_atelier_core::steps::expression::{
    MAX_EXPRESSION_NESTING_DEPTH, expression_exceeds_max_depth, expression_is_safe_to_parse,
    expression_nesting_depth,
};

#[test]
fn an_unterminated_regex_does_not_swallow_the_rest_of_the_input() {
    // Minimized from the js_ts_expression fuzz OOM (#3873). Its 1409 bytes held
    // 189 unclosed type angles and exactly one `/`, at byte 37 with no line
    // terminator after it. The `<` before that slash put the scanner in
    // regex-start position, `skip_regex` found no closing `/` and ran to the end
    // of the input, and the 183 angles behind it never reached the depth budget:
    // the guard scored the whole thing at depth 6 and passed it to OXC, which
    // speculated over every angle until it ran out of memory.
    let hidden = ["a</", &"s<".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1)].concat();

    assert!(
        expression_nesting_depth(&hidden) > MAX_EXPRESSION_NESTING_DEPTH,
        "angles behind an unterminated regex must reach the depth budget: {}",
        expression_nesting_depth(&hidden)
    );
    assert!(expression_exceeds_max_depth(&hidden));
    assert!(!expression_is_safe_to_parse(&hidden));
}

#[test]
fn a_terminated_regex_still_hides_its_contents() {
    // The other direction: a real literal must stay skipped, or ordinary code
    // with brackets inside a character class starts failing the guard.
    let literal = [
        "a = /[",
        &"(".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1),
        "]/.test(b)",
    ]
    .concat();

    assert_eq!(expression_nesting_depth(&literal), 1);
    assert!(!expression_exceeds_max_depth(&literal));
    assert!(expression_is_safe_to_parse(&literal));
}

#[test]
fn a_regex_closed_only_by_a_line_terminator_does_not_hide_what_it_spans() {
    // The `slow-unit` reproducers behind #3875 are ~26 KiB with ~6000 unclosed
    // type angles and a single `/` at byte 177. A line terminator 27 KiB later
    // "ended" that literal, so the scanner skipped everything between and saw
    // depth 22 instead of 6211. A line terminator does not close a regex — the
    // lexer errors and recovers — so the span has to be scanned.
    //
    // The trailing `/` matters: without it the scan would run out of input and
    // report the literal as unterminated anyway, so only a later slash proves
    // the terminator itself is what stops the skip.
    for terminator in ["\n", "\r", "\u{2028}", "\u{2029}"] {
        let hidden = [
            "a</",
            &"s<".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1),
            terminator,
            "/",
        ]
        .concat();

        assert!(
            expression_nesting_depth(&hidden) > MAX_EXPRESSION_NESTING_DEPTH,
            "angles spanned by an unclosed regex must reach the budget: terminator {:?} depth {}",
            terminator.escape_unicode(),
            expression_nesting_depth(&hidden)
        );
        assert!(
            !expression_is_safe_to_parse(&hidden),
            "terminator {:?}",
            terminator.escape_unicode()
        );
    }
}

#[test]
fn an_escaped_line_terminator_does_not_extend_a_regex_to_a_later_slash() {
    // A `\` before a line terminator is not an escape: the lexer still ends the
    // unterminated literal at the terminator. Consuming the pair as one escape
    // would step over LF/CR — or over the 0xE2 lead byte of LS/PS — and let a
    // later `/` "close" the literal, hiding the spanned angles again.
    for terminator in ["\n", "\r", "\u{2028}", "\u{2029}"] {
        let hidden = [
            "a</",
            &"s<".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1),
            "\\",
            terminator,
            "/",
        ]
        .concat();

        assert!(
            expression_nesting_depth(&hidden) > MAX_EXPRESSION_NESTING_DEPTH,
            "angles spanned by an escaped terminator must reach the budget: terminator {:?} depth {}",
            terminator.escape_unicode(),
            expression_nesting_depth(&hidden)
        );
        assert!(
            !expression_is_safe_to_parse(&hidden),
            "terminator {:?}",
            terminator.escape_unicode()
        );
    }
}

#[test]
fn a_line_terminator_still_ends_an_unclosed_regex() {
    // Bytes *after* the terminator were always scanned; this pins that they
    // still are.
    for terminator in ["\n", "\r", "\u{2028}", "\u{2029}"] {
        let hidden = [
            "a = /x",
            terminator,
            &"[".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1),
        ]
        .concat();

        assert!(
            expression_exceeds_max_depth(&hidden),
            "terminator {:?}",
            terminator.escape_unicode()
        );
        assert!(!expression_is_safe_to_parse(&hidden));
    }
}

#[test]
fn an_unterminated_regex_keeps_scanning_its_own_bytes() {
    // The bytes inside the never-closed literal are ordinary source to the
    // recovering lexer, so their brackets count too.
    let hidden = ["a = /", &"(".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1)].concat();

    assert_eq!(
        expression_nesting_depth(&hidden),
        MAX_EXPRESSION_NESTING_DEPTH + 1
    );
    assert!(!expression_is_safe_to_parse(&hidden));
}
