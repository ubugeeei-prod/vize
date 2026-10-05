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

#[test]
fn inline_template_origin_is_utf16_and_only_shifts_its_first_content_line() {
    for source in [
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/lsp/template-semantic-ranges-8014/InlineLf.vue.txt"
        )),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/lsp/template-semantic-ranges-8014/InlineCrlf.vue.txt"
        )),
    ] {
        let descriptor =
            vize_incremental::descriptor::parse_descriptor("Inline.vue", source).unwrap();
        let tokens = super::SemanticTokensService::tokens_from_descriptor(source, &descriptor);
        let actual = tokens
            .iter()
            .map(|t| (t.line, t.start, t.length, t.token_type, t.modifiers))
            .collect::<Vec<_>>();
        assert_eq!(
            actual,
            [
                (0, 24, 6, 9, 0),
                (0, 32, 4, 18, 0),
                (0, 37, 1, 21, 0),
                (0, 39, 5, 8, 0),
                (0, 45, 10, 17, 0),
                (0, 60, 5, 8, 0),
                (1, 8, 5, 8, 0)
            ]
        );
    }
}
