use vize_carton::expression_guard::is_expression_trailing_trivia;

#[test]
fn expression_trailing_trivia_accepts_terminated_comments() {
    assert!(is_expression_trailing_trivia(""));
    assert!(is_expression_trailing_trivia(" \t\n\r"));
    assert!(is_expression_trailing_trivia(" /* perf optimization */ "));
    assert!(is_expression_trailing_trivia(
        "\u{feff}/* one */\n/* two */"
    ));
    assert!(is_expression_trailing_trivia(" // one\r\n// two\u{2028}"));
}

#[test]
fn expression_trailing_trivia_rejects_live_tokens_and_unterminated_comments() {
    assert!(!is_expression_trailing_trivia(";"));
    assert!(!is_expression_trailing_trivia(" /* unterminated"));
    assert!(!is_expression_trailing_trivia(
        " // an EOF comment can swallow generated tokens"
    ));
}
