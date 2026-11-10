//! TSRX markup embedded in a TypeScript module (#6858).
//!
//! A `.tsrx` module is TypeScript whose expression positions may hold markup:
//! JSX-shaped elements and fragments, statement containers `@{ … }` and the
//! `@if`/`@for`/`@switch`/`@try` directives ([specification]). The module is
//! parsed **once** by the host oxc parser. Where the host reaches a markup
//! position, it hands the position to this grammar through the parser's
//! markup hook; the grammar scans the markup bytes and asks the same parser
//! to parse every embedded expression, statement, binding and head. The host
//! program keeps an empty placeholder whose span is the region's span:
//! an empty JSX fragment in expression position, an empty body for a
//! statement-container function body.
//!
//! The result is lossless: every [`Region`] and node carries exact authored
//! byte spans of the module source, and embedded code is the host's own AST.
//! Syntax and template-rule errors carry the specification's `TSRX` codes; the
//! first error ends the parse, like the pinned reference compiler.
//!
//! This is syntax only. Host semantics (the Vue profile) belong to L2.
//!
//! [specification]: https://tsrx.dev/specification

use core::cell::RefCell;

use oxc_ast::ast::{Expression, FunctionBody, Program};
use oxc_diagnostics::OxcDiagnostic;
use oxc_parser::{ParseOptions, Parser, ParserReturn};
use oxc_span::SourceType;
use vize_l0::{Allocator, Span, Vec};

mod diagnostic;
mod directive;
mod element;
mod grammar;
mod tree;

pub use tree::{
    Attribute, AttributeValue, Block, Catch, Child, Comment, CommentKind, Element, ExpressionChild,
    For, Fragment, If, IfBranch, Node, Switch, SwitchCase, TagName, Try,
};

/// Where a region opened in the host program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionSite {
    /// An expression position; the host holds an empty fragment with this span.
    Expression,
    /// A function body written as `@{ … }`; the host holds an empty body.
    FunctionBody,
}

/// One markup region the host parser handed to the grammar.
#[derive(Debug)]
pub struct Region<'a> {
    pub span: Span,
    pub site: RegionSite,
    pub node: Node<'a>,
}

/// A parsed TSRX module: the host program with placeholders, and every region.
pub struct TsrxModule<'a> {
    parsed: ParserReturn<'a>,
    regions: Vec<'a, Region<'a>>,
}

/// Parse a `.tsrx` module.
pub fn parse<'a>(allocator: &'a Allocator, source: &'a str) -> TsrxModule<'a> {
    parse_with_options(allocator, source, ParseOptions::default())
}

/// Parse a `.tsrx` module once with host parser options (for example the
/// formatter's, which drop parentheses). Regions and spans are unaffected.
pub fn parse_with_options<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: ParseOptions,
) -> TsrxModule<'a> {
    let arena = allocator.as_oxc();
    let grammar: &'a grammar::Grammar<'a> = arena.alloc(grammar::Grammar {
        regions: RefCell::new(Vec::new_in(&arena)),
    });
    let parsed = Parser::new(arena, source, SourceType::tsx())
        .with_options(options)
        .parse_with_markup(grammar);
    let mut regions = grammar.regions.replace(Vec::new_in(&arena));
    regions.sort_unstable_by_key(|region| region.span.start);
    TsrxModule { parsed, regions }
}

impl<'a> TsrxModule<'a> {
    /// The host program. Markup regions are empty placeholders with region spans.
    #[must_use]
    pub fn program(&self) -> &Program<'a> {
        &self.parsed.program
    }

    /// Every region, nested ones included, in source order.
    #[must_use]
    pub fn regions(&self) -> &[Region<'a>] {
        &self.regions
    }

    /// The region whose span is exactly `span`, as a host placeholder carries it.
    #[must_use]
    pub fn region(&self, span: Span) -> Option<&Region<'a>> {
        let index = self
            .regions
            .binary_search_by_key(&span.start, |region| region.span.start)
            .ok()?;
        self.regions.get(index).filter(|region| region.span == span)
    }

    /// The region an expression placeholder stands for: an empty fragment whose
    /// span is exactly a region opened in expression position.
    #[must_use]
    pub fn expression_region(&self, expression: &Expression<'a>) -> Option<&Region<'a>> {
        let Expression::JSXFragment(fragment) = expression else {
            return None;
        };
        if !fragment.children.is_empty() {
            return None;
        }
        let span = Span::new(fragment.span.start, fragment.span.end);
        self.region(span)
            .filter(|region| region.site == RegionSite::Expression)
    }

    /// The region a function-body placeholder stands for: an empty body whose span
    /// is exactly an `@{ … }` region opened in body position.
    #[must_use]
    pub fn body_region(&self, body: &FunctionBody<'a>) -> Option<&Region<'a>> {
        if !body.statements.is_empty() || !body.directives.is_empty() {
            return None;
        }
        let span = Span::new(body.span.start, body.span.end);
        self.region(span)
            .filter(|region| region.site == RegionSite::FunctionBody)
    }

    /// Host and grammar diagnostics in report order.
    #[must_use]
    pub fn diagnostics(&self) -> &[OxcDiagnostic] {
        &self.parsed.diagnostics
    }

    /// Whether the module parsed without errors.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        !self.parsed.panicked && !self.parsed.diagnostics.has_errors()
    }
}

#[cfg(test)]
mod tests;
