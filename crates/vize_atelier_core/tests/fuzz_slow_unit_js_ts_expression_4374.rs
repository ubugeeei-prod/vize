//! Replay of the `js_ts_expression` slow-unit reproducers from #4374.

use vize_atelier_core::steps::expression::{
    MAX_EXPRESSION_NESTING_DEPTH, expression_has_balanced_delimiters, expression_is_safe_to_parse,
    expression_nesting_depth, prefix_identifiers_in_expression, strip_typescript_from_expression,
};
use vize_atelier_core::steps::v_slot::extract_slot_prop_names;

const REPRODUCER_HEX: &str = "\
683c73644c3c5c75354c3c5c75355c7562656c423c24383c5c423c4c74796c6173716c773c285c5c5c3e32675c065c293d505c26355c75626567653c5c423c4c74796c6173716c753c285c5c5c3e32675c065c293d505c26355c756265676574\
3c64734c3c5c75355c75626561793c5c6f6e616d6573706163653c285c5c5c3e32675c065c293d505c755c3562266567653c5c423c4c74796c6173716c773c285c5c5c3e32675c065c293d505c26355c7562656765743c64734c3c5c75355c75\
4c74796c6173716c773c285c5c5c3e32675c065c293d505c26355c75626567653c5c423c4c74796c6173716c753c285c5c5c3e32675c065c293d505c26355c7562656765743c64734c3c5c75355c75626561793c5c6f6e616d6573706163653c\
285c5c5c3e32675c065c293d505c755c3562266567653c5c423c4c74796c6173716c773c285c5c5c3e32675c065c293d505c26355c7562656765743c64734c3c5c75355c75626561793c5c6f646573423c44444144446c656c44446362656179\
3c5c6f646573423c44444144446c656c4444636f6e7344444444686e44440000336373456d7070753c6144";
const OUT_OF_RANGE_IDENTIFIER_ESCAPE: &str = "\\u{110000}";

fn decode_hex(hex: &str) -> Vec<u8> {
    let bytes = hex.as_bytes();
    assert_eq!(bytes.len() % 2, 0);
    bytes
        .chunks_exact(2)
        .map(|pair| {
            let high = char::from(pair[0]).to_digit(16).unwrap();
            let low = char::from(pair[1]).to_digit(16).unwrap();
            ((high << 4) | low) as u8
        })
        .collect()
}

#[test]
fn issue_4374_slow_unit_reproducer_is_rejected_by_the_expression_guard() {
    let bytes = decode_hex(REPRODUCER_HEX);
    let source = std::str::from_utf8(&bytes).expect("fuzz input is valid UTF-8");

    assert_eq!(source.len(), 427);
    assert_eq!(source.matches('<').count(), 37);
    assert_eq!(source.matches('>').count(), 8);
    assert_eq!(source.matches('\0').count(), 2);

    assert!(expression_has_balanced_delimiters(source));
    assert!(expression_nesting_depth(source) > MAX_EXPRESSION_NESTING_DEPTH);
    assert!(!expression_is_safe_to_parse(source));
    assert_eq!(prefix_identifiers_in_expression(source).as_str(), source);
    assert_eq!(strip_typescript_from_expression(source).as_str(), source);
    assert!(extract_slot_prop_names(source).is_empty());
}

#[test]
fn closed_malformed_identifier_escapes_do_not_accumulate_across_expressions() {
    let expression = std::iter::repeat_n("factory<\\u5>()", MAX_EXPRESSION_NESTING_DEPTH + 8)
        .collect::<Vec<_>>()
        .join(" + ");

    assert!(expression_has_balanced_delimiters(&expression));
    assert!(
        expression_is_safe_to_parse(&expression),
        "closed invalid generic attempts should remain parser diagnostics, not guard rejections"
    );
}

#[test]
fn invalid_identifier_escapes_count_as_malformed_recovery_cost() {
    let count = (MAX_EXPRESSION_NESTING_DEPTH / 2) + 1;
    let invalid_scalar = ["root<", OUT_OF_RANGE_IDENTIFIER_ESCAPE]
        .concat()
        .repeat(count);
    assert_malformed_identifier_escape_rejected(&invalid_scalar, count);

    let non_identifier_start = "root<\\u0030".repeat(count);
    assert_malformed_identifier_escape_rejected(&non_identifier_start, count);
}

#[test]
fn valid_identifier_escapes_count_like_normal_type_identifiers() {
    assert!(expression_is_safe_to_parse("factory<\\u0061>()"));
    assert!(expression_is_safe_to_parse("factory<\\u{61}>()"));
}

fn assert_malformed_identifier_escape_rejected(source: &str, type_angle_count: usize) {
    assert_eq!(source.matches('<').count(), type_angle_count);
    assert!(
        type_angle_count <= MAX_EXPRESSION_NESTING_DEPTH,
        "the test must rely on malformed escape cost, not angle count alone"
    );
    assert!(expression_has_balanced_delimiters(source));
    assert!(expression_nesting_depth(source) > MAX_EXPRESSION_NESTING_DEPTH);
    assert!(!expression_is_safe_to_parse(source));
}
