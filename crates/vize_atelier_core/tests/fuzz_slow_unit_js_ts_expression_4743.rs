use vize_atelier_core::steps::expression::{
    MAX_EXPRESSION_NESTING_DEPTH, expression_is_safe_to_parse, expression_nesting_depth,
};

#[test]
fn rejects_long_prefix_operator_chain_without_nesting() {
    let source = format!("{}value", "+".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1));

    assert_eq!(expression_nesting_depth(&source), 0);
    assert!(!expression_is_safe_to_parse(&source));
}

#[test]
fn permits_ordinary_binary_plus_with_prefix_operands() {
    assert!(expression_is_safe_to_parse("a + +b + +c"));
}

#[test]
fn counts_prefix_operator_chains_across_trivia() {
    let source = format!(
        "{} value",
        std::iter::repeat_n("+ /* trivia */", MAX_EXPRESSION_NESTING_DEPTH + 1).collect::<String>()
    );

    assert!(!expression_is_safe_to_parse(&source));
}

#[test]
fn ignores_operator_runs_inside_literals_and_comments() {
    let pluses = "+".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1);

    assert!(expression_is_safe_to_parse(&format!("\"{pluses}\"")));
    assert!(expression_is_safe_to_parse(&format!(
        "value /* {pluses} */"
    )));
}

#[test]
fn counts_prefix_operator_chains_inside_template_interpolations() {
    let source = format!(
        "`literal ${{{} value}} literal`",
        "+".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1)
    );

    assert!(!expression_is_safe_to_parse(&source));
}

#[test]
fn ignores_prefix_operator_text_after_template_interpolations() {
    let pluses = "+".repeat(MAX_EXPRESSION_NESTING_DEPTH + 1);

    assert!(expression_is_safe_to_parse(&format!(
        "`${{value}} {pluses}`"
    )));
}
