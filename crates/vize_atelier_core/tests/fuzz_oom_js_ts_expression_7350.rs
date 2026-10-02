//! Exact scheduled fuzz OOM input (#7350), rejected before OXC allocation.
use vize_atelier_core::steps::expression::{
    MAX_EXPRESSION_NESTING_DEPTH, expression_is_safe_to_parse, expression_nesting_depth,
    prefix_identifiers_in_expression, strip_typescript_from_expression,
};
use vize_atelier_core::steps::v_slot::extract_slot_prop_names;

const REPRODUCER: &str = include_str!("fixtures/js_ts_expression_oom_7350.txt");

#[test]
fn escaped_slashes_do_not_hide_the_oom_input_behind_a_phantom_comment() {
    assert_eq!(REPRODUCER.len(), 1310);
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
fn malformed_slash_escapes_keep_brackets_and_type_angles_visible() {
    for lead in [r"a<\/**", r"a<\//", r"a<\/"] {
        let source = [lead, &"[".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1)].concat();
        assert!(expression_nesting_depth(&source) > MAX_EXPRESSION_NESTING_DEPTH);
        assert!(!expression_is_safe_to_parse(&source));
    }
}

#[test]
fn real_comments_and_escaped_slashes_in_literals_remain_safe() {
    for source in [
        "value /* [[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[ */ + next",
        "value // [[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[\n + next",
        r"/[\/]/.test(value)",
        r"'\/** [[' + value",
        r"`\/** [[` + value",
        r"\u0061 / 2",
    ] {
        assert!(expression_is_safe_to_parse(source), "{source:?}");
    }
}
