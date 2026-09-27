use super::has_excessive_prefix_operator_run;
use crate::expression_guard::expression_is_safe_to_parse;

#[test]
fn unary_run_refusal_starts_at_thirty_two_operators() {
    for operator in ["!", "~", "+", "-"] {
        assert!(!has_excessive_prefix_operator_run(&operator.repeat(31)));
        assert!(has_excessive_prefix_operator_run(&operator.repeat(32)));
        let at_limit = format!("{}x", operator.repeat(31));
        let over_limit = format!("{}x", operator.repeat(32));
        assert!(!has_excessive_prefix_operator_run(&at_limit));
        assert!(has_excessive_prefix_operator_run(&over_limit));
        assert!(expression_is_safe_to_parse(&at_limit));
        assert!(!expression_is_safe_to_parse(&over_limit));
    }
}

#[test]
fn paired_operators_and_prefix_keywords_keep_the_run_budget() {
    let paired_at_limit = format!("{}!x", "++".repeat(15));
    let paired_over_limit = format!("{}x", "++".repeat(16));
    assert!(!has_excessive_prefix_operator_run(&paired_at_limit));
    assert!(has_excessive_prefix_operator_run(&paired_over_limit));
    assert!(expression_is_safe_to_parse(&paired_at_limit));
    assert!(!expression_is_safe_to_parse(&paired_over_limit));
    for keyword in ["await", "delete", "typeof", "void"] {
        let at_limit = format!("{}x", format!("{keyword} ").repeat(31));
        let over_limit = format!("{}x", format!("{keyword} ").repeat(32));
        assert!(!has_excessive_prefix_operator_run(&at_limit));
        assert!(has_excessive_prefix_operator_run(&over_limit));
        assert!(expression_is_safe_to_parse(&at_limit));
        assert!(!expression_is_safe_to_parse(&over_limit));
    }
}

#[test]
fn comments_do_not_reset_a_live_prefix_run() {
    let source = format!("{}/* skipped */{}x", "!".repeat(16), "~".repeat(16));
    assert!(has_excessive_prefix_operator_run(&source));
    assert!(!expression_is_safe_to_parse(&source));
    let source = format!("{}// skipped\n{}x", "!".repeat(16), "~".repeat(16));
    assert!(has_excessive_prefix_operator_run(&source));
    assert!(!expression_is_safe_to_parse(&source));
}

#[test]
fn skipped_literal_text_does_not_count_as_prefix_operators() {
    let operators = "!".repeat(32);
    for source in [
        format!("'{operators}'"),
        format!("`{operators}`"),
        format!("/{operators}/"),
        format!("/* {operators} */x"),
        format!("// {operators}\nx"),
    ] {
        assert!(!has_excessive_prefix_operator_run(&source), "{source}");
        assert!(expression_is_safe_to_parse(&source), "{source}");
    }
}

#[test]
fn template_interpolations_keep_their_live_operator_budget() {
    let at_limit = format!("`${{{}x}}`", "!".repeat(31));
    let over_limit = format!("`${{{}x}}`", "!".repeat(32));
    assert!(!has_excessive_prefix_operator_run(&at_limit));
    assert!(has_excessive_prefix_operator_run(&over_limit));
    assert!(expression_is_safe_to_parse(&at_limit));
    assert!(!expression_is_safe_to_parse(&over_limit));
    let separate_runs = format!("{}x, {}y", "!".repeat(31), "~".repeat(31));
    assert!(!has_excessive_prefix_operator_run(&separate_runs));
    assert!(expression_is_safe_to_parse(&separate_runs));
}

#[test]
fn short_text_still_passes_through_balancing_and_lexical_guards() {
    for source in ["(x)", "[x]", "`${!値}`", "値", r"\u0061", "!値"] {
        assert!(!has_excessive_prefix_operator_run(source), "{source}");
        assert!(expression_is_safe_to_parse(source), "{source}");
    }
    for source in ["item(", "([)]", "`${(x}`", "値(", r"\u0061("] {
        assert!(!has_excessive_prefix_operator_run(source), "{source}");
        assert!(!expression_is_safe_to_parse(source), "{source}");
    }
    assert!(!expression_is_safe_to_parse(&"1".repeat(4097)));
}
