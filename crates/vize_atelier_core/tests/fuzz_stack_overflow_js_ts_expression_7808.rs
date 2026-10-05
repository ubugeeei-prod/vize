//! The exact scheduled input must remain visible to the shared expression guard.
use vize_atelier_core::steps::expression::{
    MAX_EXPRESSION_NESTING_DEPTH, expression_is_safe_to_parse, expression_nesting_depth,
    prefix_identifiers_in_expression, strip_typescript_from_expression,
};
use vize_atelier_core::steps::v_slot::extract_slot_prop_names;

const REPRODUCER: &str =
    include_str!("../../../tests/fuzz/regressions/js_ts_expression/issue-7808.input");

#[test]
fn scheduled_input_is_rejected_before_recursive_expression_parsing() {
    assert_eq!(REPRODUCER.len(), 22_933);
    assert!(expression_nesting_depth(REPRODUCER) > MAX_EXPRESSION_NESTING_DEPTH);
    assert!(!expression_is_safe_to_parse(REPRODUCER));
    assert_eq!(
        prefix_identifiers_in_expression(REPRODUCER).as_str(),
        REPRODUCER
    );
    assert_eq!(
        strip_typescript_from_expression(REPRODUCER).as_str(),
        REPRODUCER
    );
    assert!(extract_slot_prop_names(REPRODUCER).is_empty());
}

#[test]
fn even_code_backslashes_leave_the_following_template_opener_live() {
    for count in [2, 4, 16] {
        let source = [
            "\\".repeat(count).as_str(),
            "`text` ",
            "(".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1).as_str(),
            "value",
            ")".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1).as_str(),
        ]
        .concat();
        assert_eq!(
            expression_nesting_depth(&source),
            MAX_EXPRESSION_NESTING_DEPTH + 1
        );
        assert!(!expression_is_safe_to_parse(&source), "{count} backslashes");
    }
}

#[test]
fn even_code_backslashes_preserve_real_string_template_and_comment_boundaries() {
    for count in [2, 4, 16] {
        for literal in ["'(((('", "\"[[[[\"", "`{{{{`", "/* ((( */ 0", "// [[[\n0"] {
            let source = ["\\".repeat(count).as_str(), literal].concat();
            assert_eq!(expression_nesting_depth(&source), 0, "{source:?}");
            assert!(expression_is_safe_to_parse(&source), "{source:?}");
        }
    }
}

#[test]
fn odd_code_backslashes_leave_brackets_after_neutralized_quotes_live() {
    for count in [1, 3, 17] {
        for quote in ["'", "\"", "`"] {
            let source = ["\\".repeat(count).as_str(), quote, "[".repeat(32).as_str()].concat();
            assert_eq!(expression_nesting_depth(&source), 32, "{source:?}");
            assert!(!expression_is_safe_to_parse(&source));
        }
    }
}

#[test]
fn odd_code_backslashes_do_not_open_phantom_comments() {
    for count in [1, 3, 17] {
        for marker in ["//", "/*"] {
            let source = ["\\".repeat(count).as_str(), marker, "[".repeat(32).as_str()].concat();
            assert_eq!(expression_nesting_depth(&source), 32, "{source:?}");
            assert!(!expression_is_safe_to_parse(&source));
        }
    }
}

#[test]
fn normal_unicode_escapes_and_literal_backslashes_keep_their_existing_behavior() {
    for source in [
        r"\u0061 + b",
        r"'\\(' + value",
        r"`\\[${value}`",
        r"/[\\(]/u.test(value)",
    ] {
        assert!(expression_is_safe_to_parse(source), "{source:?}");
    }
}
