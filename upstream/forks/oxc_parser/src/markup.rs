//! Host-embedded markup: markup written inside a JS/TS program.
//!
//! This is the reverse of embedding script in markup. The host grammar stays
//! the owner of every expression position. When it reaches a position where a
//! registered [`MarkupGrammar`] opens a region, it hands that position to the
//! grammar, which parses the region and returns its end. The host keeps an
//! empty placeholder whose span is exactly the region. While parsing, the
//! grammar asks this same parser, through a [`MarkupCursor`], to parse every
//! embedded expression, statement, binding and head, so those nodes are
//! ordinary AST nodes in the same arena with original spans.
//!
//! Positions that can open a region:
//! - a `<` where JSX is enabled ([`MarkupSite::Expression`]);
//! - an `@` at the start of an expression or statement, when the grammar opens
//!   it ([`MarkupSite::Expression`]);
//! - an `@` where a function body would start ([`MarkupSite::FunctionBody`]).
//!
//! While a grammar is registered, a `<` that starts a line and is followed by a
//! tag-like character ends the preceding expression (so automatic semicolon
//! insertion separates `const a = b` from a following `<tag>`), matching the
//! line-start rule of markup-in-script languages.

use oxc_allocator::{Allocator, ArenaBox, ArenaVec, GetAllocator};
use oxc_ast::ast::*;
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

use crate::{
    Context, Parser, ParserConfig as Config, ParserImpl, ParserReturn, StatementContext,
    config::NoTokensParserConfig,
    diagnostics::{self, ParserDiagnostic},
    js::{VariableDeclarationParent, grammar::CoverGrammar},
    lexer::{Kind, Token},
};

/// Where a markup region opens in the host grammar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkupSite {
    /// An expression (or expression statement) position.
    Expression,
    /// A function, method or arrow body position.
    FunctionBody,
}

/// A markup grammar embedded in host JS/TS.
///
/// The grammar is shared (`&self`) so a region may re-enter the host parser,
/// which may open nested regions with the same grammar.
pub trait MarkupGrammar<'a> {
    /// Whether a region opens at `offset` (the start of an `@` token) for `site`.
    /// A `<` in a JSX-enabled profile always opens an expression region.
    fn opens(&self, source: &'a str, offset: u32, site: MarkupSite) -> bool;

    /// Parse one region that starts at `cursor.token()`, and return the offset where
    /// it ends. Errors are reported with [`MarkupCursor::fatal`].
    fn region(&self, cursor: &mut MarkupCursor<'_, 'a>, site: MarkupSite) -> u32;
}

/// The host operations a markup grammar may use inside one region.
pub struct MarkupCursor<'p, 'a> {
    host: &'p mut dyn MarkupHost<'a>,
}

trait MarkupHost<'a> {
    fn source(&self) -> &'a str;
    fn allocator(&self) -> &'a Allocator;
    fn token(&self) -> Token;
    fn seat(&mut self, offset: u32) -> bool;
    fn bump(&mut self);
    fn expression(&mut self) -> Expression<'a>;
    fn sequence(&mut self) -> Expression<'a>;
    fn statement(&mut self) -> Statement<'a>;
    fn for_of_head(&mut self) -> Option<(ForStatementLeft<'a>, Expression<'a>)>;
    fn binding_identifier(&mut self) -> BindingIdentifier<'a>;
    fn formal_parameters(&mut self) -> ArenaBox<'a, FormalParameters<'a>>;
    fn fatal(&mut self, diagnostic: OxcDiagnostic);
    fn failed(&self) -> bool;
}

impl<'a> MarkupCursor<'_, 'a> {
    /// The whole host source.
    pub fn source(&self) -> &'a str {
        self.host.source()
    }

    /// The arena of the host AST.
    pub fn allocator(&self) -> &'a Allocator {
        self.host.allocator()
    }

    /// The current host token.
    pub fn token(&self) -> Token {
        self.host.token()
    }

    /// Move the host to `offset` and read the token there (after trivia).
    /// Returns `false` (and reports a fatal error) when `offset` is not a
    /// character boundary of the source.
    pub fn seat(&mut self, offset: u32) -> bool {
        self.host.seat(offset)
    }

    /// Advance past the current host token.
    pub fn bump(&mut self) {
        self.host.bump();
    }

    /// Parse an `AssignmentExpression` at the current token, with `in` allowed.
    pub fn expression(&mut self) -> Expression<'a> {
        self.host.expression()
    }

    /// Parse an `Expression` (a comma sequence) at the current token.
    pub fn sequence(&mut self) -> Expression<'a> {
        self.host.sequence()
    }

    /// Parse one `StatementListItem` at the current token.
    pub fn statement(&mut self) -> Statement<'a> {
        self.host.statement()
    }

    /// Parse `( ForDeclaration of AssignmentExpression` at the current `(`, leaving
    /// the token after the iterable (a `;` clause or the closing `)`).
    /// Returns `None`, leaving the token after the left side, when the head is not a for-of head.
    pub fn for_of_head(&mut self) -> Option<(ForStatementLeft<'a>, Expression<'a>)> {
        self.host.for_of_head()
    }

    /// Parse a `BindingIdentifier` at the current token.
    pub fn binding_identifier(&mut self) -> BindingIdentifier<'a> {
        self.host.binding_identifier()
    }

    /// Parse parenthesized `FormalParameters` at the current `(`.
    pub fn formal_parameters(&mut self) -> ArenaBox<'a, FormalParameters<'a>> {
        self.host.formal_parameters()
    }

    /// Report an error that ends the parse.
    pub fn fatal(&mut self, diagnostic: OxcDiagnostic) {
        self.host.fatal(diagnostic);
    }

    /// Whether the parse has already failed fatally.
    pub fn failed(&self) -> bool {
        self.host.failed()
    }
}

impl<'a, C: Config> MarkupHost<'a> for ParserImpl<'a, C> {
    fn source(&self) -> &'a str {
        self.source_text
    }

    fn allocator(&self) -> &'a Allocator {
        GetAllocator::allocator(self)
    }

    fn token(&self) -> Token {
        self.token
    }

    fn seat(&mut self, offset: u32) -> bool {
        self.seat_at(offset)
    }

    fn bump(&mut self) {
        self.bump_any();
    }

    fn expression(&mut self) -> Expression<'a> {
        self.context_add(Context::In, ParserImpl::parse_assignment_expression_or_higher)
    }

    fn sequence(&mut self) -> Expression<'a> {
        self.context_add(Context::In, ParserImpl::parse_expr)
    }

    fn statement(&mut self) -> Statement<'a> {
        self.parse_statement_list_item(StatementContext::StatementList)
    }

    fn for_of_head(&mut self) -> Option<(ForStatementLeft<'a>, Expression<'a>)> {
        self.parse_markup_for_of_head()
    }

    fn binding_identifier(&mut self) -> BindingIdentifier<'a> {
        self.parse_binding_identifier()
    }

    fn formal_parameters(&mut self) -> ArenaBox<'a, FormalParameters<'a>> {
        let (_, parameters) = self.parse_formal_parameters(
            crate::js::FunctionKind::Expression,
            FormalParameterKind::FormalParameter,
        );
        parameters
    }

    fn fatal(&mut self, diagnostic: OxcDiagnostic) {
        self.set_fatal_error(ParserDiagnostic::Eager(Box::new(diagnostic)));
    }

    fn failed(&self) -> bool {
        self.fatal_error.is_some()
    }
}

impl<'a, C: Config> ParserImpl<'a, C> {
    /// Move to `offset` and make the token there current.
    pub(crate) fn seat_at(&mut self, offset: u32) -> bool {
        if let Some(token) = self.lexer.seat(offset) {
            self.prev_token_end = offset;
            self.token = token;
            true
        } else {
            self.set_fatal_error(diagnostics::unexpected_token(Span::empty(offset)));
            false
        }
    }

    /// Whether the current `@` opens a markup region for `site`.
    #[inline]
    pub(crate) fn at_markup_directive(&self, site: MarkupSite) -> bool {
        C::MARKUP && self.at(Kind::At) && self.markup_opens(site)
    }

    #[cold]
    #[inline(never)]
    fn markup_opens(&self, site: MarkupSite) -> bool {
        self.markup
            .is_some_and(|grammar| grammar.opens(self.source_text, self.cur_token().start(), site))
    }

    /// Whether the current `<` starts a line and a tag while a grammar is registered.
    #[inline]
    pub(crate) fn at_markup_line_start_tag(&self) -> bool {
        C::MARKUP && self.markup_line_start_tag()
    }

    #[cold]
    #[inline(never)]
    fn markup_line_start_tag(&self) -> bool {
        let token = self.cur_token();
        token.kind() == Kind::LAngle
            && token.is_on_new_line()
            && self.source_text.as_bytes().get(token.end() as usize).is_some_and(|byte| {
                byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$' | b'>' | b'{')
            })
    }

    /// Run the registered grammar on the region at the current token; return its end.
    fn run_markup(&mut self, site: MarkupSite) -> u32 {
        let start = self.cur_token().start();
        let Some(grammar) = self.markup else {
            self.set_unexpected();
            return start;
        };
        let end = grammar.region(&mut MarkupCursor { host: self }, site);
        if self.fatal_error.is_none() && (end <= start || !self.seat_at(end)) {
            self.set_fatal_error(diagnostics::unexpected_token(Span::empty(start)));
        }
        end.max(start)
    }

    /// An expression-position region, kept as an empty fragment with the region's span.
    #[cold]
    #[inline(never)]
    pub(crate) fn parse_markup_expression(&mut self) -> Expression<'a> {
        let start = self.cur_token().start();
        let end = self.run_markup(MarkupSite::Expression);
        let opening = JSXOpeningFragment::new(Span::empty(start), self);
        let closing = JSXClosingFragment::new(Span::empty(end), self);
        Expression::JSXFragment(JSXFragment::boxed(
            Span::new(start, end),
            opening,
            ArenaVec::new_in(self),
            closing,
            self,
        ))
    }

    /// A function-body region, kept as an empty body with the region's span.
    #[cold]
    #[inline(never)]
    pub(crate) fn parse_markup_function_body(&mut self) -> ArenaBox<'a, FunctionBody<'a>> {
        let start = self.cur_token().start();
        let end = self.context(Context::Return, Context::TopLevel, |p| {
            p.run_markup(MarkupSite::FunctionBody)
        });
        FunctionBody::boxed(
            Span::new(start, end),
            ArenaVec::new_in(self),
            ArenaVec::new_in(self),
            self,
        )
    }

    fn parse_markup_for_of_head(&mut self) -> Option<(ForStatementLeft<'a>, Expression<'a>)> {
        self.expect(Kind::LParen);
        let declaration_kind = match self.cur_kind() {
            Kind::Const => Some(VariableDeclarationKind::Const),
            Kind::Let => Some(VariableDeclarationKind::Let),
            Kind::Var => Some(VariableDeclarationKind::Var),
            _ => None,
        };
        let left = if let Some(kind) = declaration_kind {
            let start = self.start_span();
            self.bump_any();
            let declaration = self.context_remove(Context::In, |p| {
                p.parse_variable_declaration(start, kind, VariableDeclarationParent::For, false)
            });
            ForStatementLeft::VariableDeclaration(declaration)
        } else {
            let target =
                self.context_remove(Context::In, ParserImpl::parse_lhs_expression_or_higher);
            ForStatementLeft::from(AssignmentTarget::cover(target, self))
        };
        if !self.at(Kind::Of) {
            return None;
        }
        self.bump_any();
        let right =
            self.context_add(Context::In, ParserImpl::parse_assignment_expression_or_higher);
        Some((left, right))
    }
}

impl<'a> Parser<'a, NoTokensParserConfig> {
    /// Parse a program whose markup regions belong to `grammar`.
    ///
    /// The result is an ordinary [`ParserReturn`]. It never mints
    /// [`crate::ProgramObservation`] admission: its program contains
    /// placeholders that only the grammar's own artifact can interpret.
    pub fn parse_with_markup(self, grammar: &'a dyn MarkupGrammar<'a>) -> ParserReturn<'a> {
        crate::parse_with_markup(
            self.allocator,
            self.source_text,
            self.source_type,
            self.options,
            grammar,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use oxc_allocator::Allocator;
    use oxc_ast::ast::{Expression, Statement};
    use oxc_span::{GetSpan, SourceType};

    use super::{MarkupCursor, MarkupGrammar, MarkupSite};
    use crate::{Kind, Parser};

    type Recorded = Vec<(MarkupSite, u32, u32, String)>;

    /// `<{ expression }>` regions and `@{ statements }` bodies.
    #[derive(Default)]
    struct Toy {
        regions: RefCell<Recorded>,
    }

    impl<'a> MarkupGrammar<'a> for Toy {
        fn opens(&self, source: &'a str, offset: u32, _: MarkupSite) -> bool {
            source.as_bytes().get(offset as usize + 1) == Some(&b'{')
        }

        fn region(&self, cursor: &mut MarkupCursor<'_, 'a>, site: MarkupSite) -> u32 {
            let start = cursor.token().start();
            cursor.seat(start + 2);
            let (inner, end) = if cursor.source().as_bytes()[start as usize] == b'<' {
                let expression = cursor.expression();
                let close = cursor.token();
                assert_eq!(close.kind(), Kind::RCurly);
                (expression.span().source_text(cursor.source()).to_string(), close.end() + 1)
            } else {
                let mut statements = Vec::new();
                while cursor.token().kind() != Kind::RCurly {
                    let statement = cursor.statement();
                    statements.push(statement.span().source_text(cursor.source()).to_string());
                }
                (statements.join("|"), cursor.token().end())
            };
            self.regions.borrow_mut().push((site, start, end, inner));
            end
        }
    }

    fn parse(source: &str) -> (Recorded, usize, Option<(u32, u32)>) {
        let toy = Toy::default();
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, source, SourceType::tsx()).parse_with_markup(&toy);
        assert!(parsed.diagnostics.is_empty(), "{source}: {:?}", parsed.diagnostics);
        let placeholder = match parsed.program.body.first() {
            Some(Statement::VariableDeclaration(declaration)) => {
                match declaration.declarations[0].init.as_ref() {
                    Some(Expression::JSXFragment(fragment)) => {
                        assert!(fragment.children.is_empty());
                        Some((fragment.span.start, fragment.span.end))
                    }
                    _ => None,
                }
            }
            Some(Statement::FunctionDeclaration(function)) => {
                let body = function.body.as_ref().unwrap();
                assert!(body.statements.is_empty());
                Some((body.span.start, body.span.end))
            }
            _ => None,
        };
        let regions = toy.regions.borrow().clone();
        (regions, parsed.program.body.len(), placeholder)
    }

    #[test]
    fn regions_keep_exact_placeholders_and_reenter_the_host() {
        let (regions, statements, placeholder) = parse("const a = <{b + [<{c}>][0]}>;");
        assert_eq!(statements, 1);
        assert_eq!(placeholder, Some((10, 28)));
        assert_eq!(
            regions,
            [
                (MarkupSite::Expression, 17, 22, "c".to_string()),
                (MarkupSite::Expression, 10, 28, "b + [<{c}>][0]".to_string()),
            ]
        );
    }

    #[test]
    fn function_bodies_and_statement_starts_open_regions() {
        let (regions, _, placeholder) = parse("function f() @{ const x = 1; g(x) }");
        assert_eq!(placeholder, Some((13, 35)));
        assert_eq!(regions, [(MarkupSite::FunctionBody, 13, 35, "const x = 1;|g(x)".to_string())]);
        let (regions, statements, _) = parse("let v;\n@{ v = 1 }\n@dec class K {}");
        assert_eq!(statements, 3);
        assert_eq!(regions, [(MarkupSite::Expression, 7, 17, "v = 1".to_string())]);
    }

    #[test]
    fn a_line_start_tag_ends_the_previous_expression() {
        let (regions, statements, _) = parse("const a = b\n<{c}>\nconst d = e < f;");
        assert_eq!(statements, 3);
        assert_eq!(regions, [(MarkupSite::Expression, 12, 17, "c".to_string())]);
    }
}
