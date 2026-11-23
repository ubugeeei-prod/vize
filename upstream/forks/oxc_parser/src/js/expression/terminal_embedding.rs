//! Parse the fixed wrapper's sole statement when the ordinary lexer proves
//! that one primary fills the entire expression. Every occurrence still owns
//! its own parser, source, AST, module record and diagnostics.
use super::{Config, Context, Expression, Kind, ParserImpl};
use oxc_ast::ast::Statement;

impl<'a, C: Config> ParserImpl<'a, C> {
    pub(crate) fn terminal_embedding_statement(&mut self) -> Option<Statement<'a>> {
        if !self.at(Kind::LParen) || self.cur_token().start() != 0 {
            return None;
        }
        let token = self.lexer.peek_token();
        // The private input owns "(\n" + complete content + "\n)". No other
        // token can precede the closing parenthesis or follow the statement.
        if token.start() != 2
            || token.end() as usize + 2 != self.source_text.len()
            || !matches!(token.kind(), Kind::Ident | Kind::Decimal)
        {
            return None;
        }
        let start = self.start_span();
        self.bump(Kind::LParen);
        let primary = self.context(Context::In, Context::Decorator, Self::parse_primary_expression);
        self.expect(Kind::RParen);
        let expression = if self.options.preserve_parens {
            Expression::new_parenthesized_expression(self.end_span(start), primary, self)
        } else {
            primary
        };
        Some(self.parse_expression_statement(start, expression))
    }
}
