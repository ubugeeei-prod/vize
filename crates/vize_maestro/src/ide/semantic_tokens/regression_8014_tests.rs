use super::{TokenType, expressions::tokenize_expression};

#[test]
fn template_literal_substitutions_and_comments_have_complete_single_line_utf16_tokens() {
    for newline in ["\n", "\r\n"] {
        let source = vize_l0::cstr!("`😀-${{open /* note */ ? 'down' : 'right'}}{newline}尾`");
        let mut tokens = Vec::new();
        tokenize_expression(&source, &source, 0, 0, &mut tokens);
        let actual = tokens
            .iter()
            .map(|t| (t.line, t.start, t.length, t.token_type, t.modifiers))
            .collect::<Vec<_>>();
        assert_eq!(
            actual,
            [
                (0, 0, 4, TokenType::String as u32, 0),
                (0, 6, 4, TokenType::Variable as u32, 0),
                (0, 11, 10, TokenType::Comment as u32, 0),
                (0, 22, 1, TokenType::Operator as u32, 0),
                (0, 24, 6, TokenType::String as u32, 0),
                (0, 31, 1, TokenType::Operator as u32, 0),
                (0, 33, 7, TokenType::String as u32, 0),
                (0, 40, 1, TokenType::String as u32, 0),
                (1, 0, 2, TokenType::String as u32, 0)
            ]
        );
    }
}

#[test]
fn unaffected_expression_operators_identifiers_and_quote_tokens_are_exact() {
    let source = "obj.value + run(1.5e-2, '😀') ?? count";
    let mut tokens = Vec::new();
    tokenize_expression(source, source, 0, 0, &mut tokens);
    let actual = tokens
        .iter()
        .map(|t| (t.start, t.length, t.token_type, t.modifiers))
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        [
            (0, 3, 8, 0),
            (4, 5, 9, 0),
            (10, 1, 21, 0),
            (12, 3, 12, 0),
            (16, 6, 19, 0),
            (24, 4, 18, 0),
            (30, 2, 21, 0),
            (33, 5, 8, 0)
        ]
    );
}
