//! Resolve the fixed wrapper's two unambiguous productions from its actual
//! lexer token. All inner nodes, parentheses, statements, spans and parser
//! observations are built by the ordinary production that owns them.
use super::{Config, Expression, Kind, ParserImpl};

impl<'a, C: Config> ParserImpl<'a, C> {
    #[inline]
    pub(super) fn terminal_embedding_expression(&mut self) -> Option<Expression<'a>> {
        let outer = self.at(Kind::LParen) && self.cur_token().start() == 0;
        let token = if outer { self.lexer.peek_token() } else { self.cur_token() };
        // The private input owns "(\n" + complete content + "\n)". A single
        // lexed terminal filling that content leaves no suffix for an arrow,
        // postfix operator, type annotation, binary or assignment expression.
        if token.start() != 2
            || token.end() as usize + 2 != self.source_text.len()
            || !matches!(token.kind(), Kind::Ident | Kind::Decimal)
        {
            return None;
        }
        Some(if outer {
            self.parse_parenthesized_expression()
        } else {
            self.parse_primary_expression()
        })
    }
}
