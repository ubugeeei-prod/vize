//! Safe consuming handoff of a retained expression and its observations.

use alloc::boxed::Box;
use oxc_ast::ast::{Comment, Expression, Statement};
use oxc_diagnostics::Diagnostics;
use oxc_span::SourceType;
use vize_l0::{Allocator, Span};

use super::{
    CommentView, DiagnosticView, EmbedHole, EmbedSource, Grammar, NativeSyntax, Shape, SourceError,
    coordinates::Coordinates,
};

/// The same parsed root moved into the shared arena, plus owned observations.
///
/// Its root can outlive this observation owner at the allocator's lifetime.
/// Diagnostics remain normally owned and dropped, never parked in arena bytes.
/// AST locations still use checked wrapper/source projections, and comments
/// remain from the original single parse. A local hole exposes no recovery AST.
pub struct RetainedExpression<'a> {
    grammar: Grammar,
    source_type: SourceType,
    coordinates: Coordinates<'a>,
    expression: Option<&'a Expression<'a>>,
    comments: &'a [Comment],
    diagnostics: Diagnostics,
    hole: Option<EmbedHole>,
}

impl core::fmt::Debug for RetainedExpression<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RetainedExpression")
            .field("grammar", &self.grammar)
            .field("source", &self.source())
            .field("hole", &self.hole)
            .field("diagnostic_count", &self.diagnostics.len())
            .finish_non_exhaustive()
    }
}

impl<'a> RetainedExpression<'a> {
    #[must_use]
    pub const fn grammar(&self) -> Grammar {
        self.grammar
    }

    #[must_use]
    pub const fn source_type(&self) -> SourceType {
        self.source_type
    }

    #[must_use]
    pub const fn source(&self) -> EmbedSource<'a> {
        self.coordinates.source
    }

    /// Generated prefix bytes in the retained AST's parser coordinates.
    #[must_use]
    pub const fn parser_prefix(&self) -> u32 {
        self.coordinates.prefix
    }

    #[must_use]
    pub const fn expression(&self) -> Option<&'a Expression<'a>> {
        self.expression
    }

    #[must_use]
    pub const fn hole(&self) -> Option<EmbedHole> {
        self.hole
    }

    pub fn comments(&self) -> impl Iterator<Item = CommentView<'_, 'a>> {
        self.comments.iter().map(|comment| CommentView {
            comment,
            coordinates: self.coordinates,
        })
    }

    pub fn diagnostics(&self) -> impl Iterator<Item = DiagnosticView<'_, 'a>> {
        self.diagnostics.iter().map(|diagnostic| DiagnosticView {
            diagnostic,
            coordinates: self.coordinates,
        })
    }

    pub fn decoded_span(&self, span: oxc_span::Span) -> Result<Span, SourceError> {
        self.coordinates.decoded_span(span)
    }

    pub fn authored_span(&self, span: oxc_span::Span) -> Result<Span, SourceError> {
        self.source().authored_span(self.decoded_span(span)?)
    }
}

pub(super) fn into_expression<'a>(
    syntax: NativeSyntax<'a>,
    allocator: &'a Allocator,
) -> Result<RetainedExpression<'a>, Box<NativeSyntax<'a>>> {
    if syntax.grammar.shape != Shape::Expr {
        return Err(Box::new(syntax));
    }
    let NativeSyntax {
        grammar,
        source_type,
        coordinates,
        program,
        observation: _,
        diagnostics,
        mut hole,
    } = syntax;
    let mut expression = None;
    let mut comments = &[][..];
    if let Some(mut program) = program {
        comments = program.comments.into_arena_slice();
        if hole.is_none()
            && program.body.len() == 1
            && let Some(Statement::ExpressionStatement(statement)) = program.body.pop()
            && let Expression::ParenthesizedExpression(wrapper) = statement.unbox().expression
        {
            expression = Some(&*allocator.alloc(wrapper.unbox().expression));
        }
    }
    if hole.is_none() && expression.is_none() {
        hole = Some(EmbedHole::InvalidExpressionShape);
    }
    Ok(RetainedExpression {
        grammar,
        source_type,
        coordinates,
        expression,
        comments,
        diagnostics,
        hole,
    })
}

#[cfg(test)]
mod tests;
