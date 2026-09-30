use vize_carton::expression_guard::is_expression_trailing_trivia;

#[test]
fn expression_trailing_trivia_accepts_whitespace_and_closed_block_comments() {
    assert!(is_expression_trailing_trivia(""));
    assert!(is_expression_trailing_trivia(" \t\n\r"));
    assert!(is_expression_trailing_trivia(" /* perf optimization */ "));
    assert!(is_expression_trailing_trivia(
        "\u{feff}/* one */\n/* two */"
    ));
}

#[test]
fn expression_trailing_trivia_rejects_live_tokens_and_line_comments() {
    assert!(!is_expression_trailing_trivia(";"));
    assert!(!is_expression_trailing_trivia(" /* unterminated"));
    assert!(!is_expression_trailing_trivia(
        " // comments can swallow generated tokens"
    ));
}
