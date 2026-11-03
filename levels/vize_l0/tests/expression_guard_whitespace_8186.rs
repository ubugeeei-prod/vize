//! Keep regex goals and unary depth unchanged across ECMAScript ASCII trivia.

#![expect(
    clippy::disallowed_macros,
    reason = "boundary laws use formatted complete source"
)]

use vize_l0::expression_guard::{
    MAX_EXPRESSION_NESTING_DEPTH, expression_is_safe_to_parse, expression_nesting_depth,
};

const ORIGINAL_A: &str = include_str!(
    "../../../tests/fuzz/regressions/js_ts_expression/slow-unit-a117c12decb552d05f038566e9a966039d91acb1"
);
const ORIGINAL_B: &str = include_str!(
    "../../../tests/fuzz/regressions/js_ts_expression/slow-unit-20ec7d2257bd09bef36390f4003322c72eb32f0a"
);

#[test]
fn original_slow_units_are_rejected_before_recursive_parsing() {
    for (source, bytes) in [(ORIGINAL_A, 23_424), (ORIGINAL_B, 23_436)] {
        assert_eq!(source.len(), bytes);
        assert!(expression_nesting_depth(source) > MAX_EXPRESSION_NESTING_DEPTH);
        assert!(!expression_is_safe_to_parse(source));
    }
}

#[test]
fn ascii_trivia_preserves_the_regex_goal_and_live_delimiter_depth() {
    for whitespace in [' ', '\t', '\n', '\r', '\u{b}', '\u{c}'] {
        // The backtick belongs to a regex, so it must not hide the live parens.
        let source = format!(
            "!{whitespace}/`/ + {}value{}",
            "(".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1),
            ")".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1),
        );
        assert_eq!(
            expression_nesting_depth(&source),
            MAX_EXPRESSION_NESTING_DEPTH + 1
        );
        assert!(!expression_is_safe_to_parse(&source));

        // Delimiters inside a real regex remain lexical content.
        let regex = format!("!{whitespace}/[{}]/.test(value)", "(".repeat(64));
        assert_eq!(expression_nesting_depth(&regex), 1);
        assert!(expression_is_safe_to_parse(&regex));

        // A postfix non-null assertion keeps the following slash as division.
        let division = format!("value!{whitespace}/ `[{}]`", "(".repeat(64));
        assert_eq!(expression_nesting_depth(&division), 0);
        assert!(expression_is_safe_to_parse(&division));
    }
}

#[test]
fn ascii_trivia_does_not_reset_the_prefix_operator_budget() {
    for whitespace in [' ', '\t', '\n', '\r', '\u{b}', '\u{c}'] {
        let prefix = format!("!{whitespace}");
        let allowed = format!("{}value", prefix.repeat(MAX_EXPRESSION_NESTING_DEPTH));
        let rejected = format!("{}value", prefix.repeat(MAX_EXPRESSION_NESTING_DEPTH + 1));
        assert!(expression_is_safe_to_parse(&allowed));
        assert!(!expression_is_safe_to_parse(&rejected));
    }
}
