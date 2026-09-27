use vize_carton::expression_guard::{
    MAX_EXPRESSION_NESTING_DEPTH, expression_is_safe_to_parse, is_expression_trailing_trivia,
};

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

#[test]
fn ascii_word_safety_keeps_identifier_and_numeric_boundaries() {
    for word in [
        "item", "items", "key", "index", "_", "$", "$event", "x1", "typeof",
    ] {
        assert!(expression_is_safe_to_parse(word), "{word}");
    }
    // Digits belong to this identifier; they are not an oversized numeric token.
    assert!(expression_is_safe_to_parse(&format!(
        "x{}",
        "1".repeat(4097)
    )));
    assert!(expression_is_safe_to_parse(&"1".repeat(4096)));
    assert!(!expression_is_safe_to_parse(&"1".repeat(4097)));
}

#[test]
fn identifier_prefixes_cannot_hide_guard_refused_syntax() {
    let safe_depth = MAX_EXPRESSION_NESTING_DEPTH;
    let boundary = format!("{}x{}", "(".repeat(safe_depth), ")".repeat(safe_depth));
    assert!(expression_is_safe_to_parse(&boundary));
    let depth = MAX_EXPRESSION_NESTING_DEPTH + 1;
    let nested = format!("word{}x{}", "(".repeat(depth), ")".repeat(depth));
    assert!(!expression_is_safe_to_parse(&nested));
    let symbolic_prefixes = format!("{}x", "!".repeat(depth));
    assert!(!expression_is_safe_to_parse(&symbolic_prefixes));
    let keyword_prefixes = format!("{}x", "typeof ".repeat(depth));
    assert!(!expression_is_safe_to_parse(&keyword_prefixes));
    assert!(!expression_is_safe_to_parse("item("));
    assert!(!expression_is_safe_to_parse(r"item\u0061("));
    assert!(expression_is_safe_to_parse("item.value"));
    assert!(expression_is_safe_to_parse("item /* tail */"));
    assert!(expression_is_safe_to_parse("値"));
    assert!(expression_is_safe_to_parse(r"\u0061"));
}
