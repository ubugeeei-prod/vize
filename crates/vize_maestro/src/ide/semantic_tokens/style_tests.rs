//! `v-bind()` highlighting inside `<style>` blocks of a whole SFC.

use tower_lsp::lsp_types::{SemanticTokensResult, Url};

use super::{SemanticTokensService, TokenType};

/// `(line, start, text, type)` for every token on a `<style>` line.
fn style_tokens(content: &str) -> Vec<(u32, u32, String, u32)> {
    let uri = Url::parse("file:///StyleTokens.vue").unwrap();
    let Some(SemanticTokensResult::Tokens(tokens)) =
        SemanticTokensService::get_tokens(&crate::server::ServerState::new(), content, &uri)
    else {
        panic!("expected full semantic tokens");
    };
    let lines: Vec<&str> = content.lines().collect();
    let style_start = lines
        .iter()
        .position(|line| line.starts_with("<style"))
        .unwrap() as u32;
    let (mut line, mut start) = (0, 0);
    let mut out = Vec::new();
    for token in tokens.data {
        line += token.delta_line;
        start = if token.delta_line == 0 {
            start + token.delta_start
        } else {
            token.delta_start
        };
        if line > style_start {
            let text = lines[line as usize]
                .chars()
                .skip(start as usize)
                .take(token.length as usize)
                .collect();
            out.push((line, start, text, token.token_type));
        }
    }
    out
}

const F: u32 = TokenType::Function as u32;
const V: u32 = TokenType::Variable as u32;

#[test]
fn script_bindings_and_quoted_expressions_in_v_bind_are_highlighted() {
    let content = "<script setup>\nconst color = 'red'\n</script>\n<style>\n.a { color: v-bind(color); }\n.b { width: v-bind( 'size.w' ) }\n</style>\n";
    assert_eq!(
        style_tokens(content),
        [
            (4, 12, "v-bind".to_owned(), F),
            (4, 19, "color".to_owned(), V),
            (5, 12, "v-bind".to_owned(), F),
            (5, 21, "size.w".to_owned(), V),
        ]
    );
}

#[test]
fn v_bind_inside_comments_and_strings_is_css_text() {
    let content = "<style>\n/* v-bind(ignored) */\n.a { color: v-bind(color) }\n.b::after { content: \"v-bind(no)\" }\n</style>\n";
    assert_eq!(
        style_tokens(content),
        [
            (2, 12, "v-bind".to_owned(), F),
            (2, 19, "color".to_owned(), V)
        ]
    );
}

#[test]
fn a_quoted_argument_ends_at_its_closing_quote() {
    let content = "<style>\n.a { width: v-bind('calc(x)') ; color: v-bind(c) }\n</style>\n";
    assert_eq!(
        style_tokens(content),
        [
            (1, 12, "v-bind".to_owned(), F),
            (1, 20, "calc(x)".to_owned(), V),
            (1, 39, "v-bind".to_owned(), F),
            (1, 46, "c".to_owned(), V),
        ]
    );
}

#[test]
fn non_ascii_selectors_and_bindings_use_utf16_columns() {
    let content = "<style>\n.日本 { color: v-bind(色) }\n</style>\n";
    assert_eq!(
        style_tokens(content),
        [(1, 13, "v-bind".to_owned(), F), (1, 20, "色".to_owned(), V)]
    );
}
