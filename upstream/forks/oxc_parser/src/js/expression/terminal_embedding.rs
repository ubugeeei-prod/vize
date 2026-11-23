//! Parse the fixed wrapper's sole statement when the ordinary lexer proves
//! that one primary fills the entire expression. Every occurrence still owns
//! its own parser, source, AST, module record and diagnostics.
use super::{Config, Context, Expression, Kind, ParserImpl};
use crate::embedding::terminal::TerminalExpression;
use core::any::TypeId;
use oxc_allocator::ArenaVec;
use oxc_ast::ast::{Directive, Statement};

impl<'a, C: Config> ParserImpl<'a, C> {
    /// Only the fixed wrapper can select this grammar, at the program root.
    pub(crate) fn parse_program_body_with_terminal_operand(
        &mut self,
    ) -> (ArenaVec<'a, Directive<'a>>, ArenaVec<'a, Statement<'a>>) {
        if TypeId::of::<C>() == TypeId::of::<TerminalExpression>()
            && let Some(statement) = self.terminal_embedding_statement()
        {
            let mut statements = ArenaVec::with_capacity_in(1, self);
            statements.push(statement);
            return (ArenaVec::new_in(self), statements);
        }
        self.parse_directives_and_statements(false)
    }

    fn terminal_embedding_statement(&mut self) -> Option<Statement<'a>> {
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
