//! Consuming the original parser-owned expression and its normal observations.

use alloc::boxed::Box;
use oxc_ast::ast::Expression;
use oxc_diagnostics::Diagnostics;
use oxc_parser::{AdmittedExpression, ExpressionObservation};
use oxc_span::SourceType;
use vize_l0::Span;

use super::{
    CommentView, DiagnosticView, EmbedHole, EmbedSource, Grammar, NativeSyntax, Shape, SourceError,
    coordinates::Coordinates,
};

/// The same arena root, authenticated by its original ordinary parser owner.
pub struct RetainedExpression<'a> {
    grammar: Grammar,
    source_type: SourceType,
    coordinates: Coordinates<'a>,
    observation: Option<ExpressionObservation<'a>>,
    diagnostics: Diagnostics,
    hole: Option<EmbedHole>,
}

impl core::fmt::Debug for RetainedExpression<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RetainedExpression")
            .field("grammar", &self.grammar)
            .field("source", &self.source())
            .field("hole", &self.hole)
            .field("diagnostic_count", &self.diagnostics().count())
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
    #[must_use]
    pub const fn parser_prefix(&self) -> u32 {
        self.coordinates.prefix
    }
    #[must_use]
    pub fn admitted_expression(&self) -> Option<AdmittedExpression<'_, 'a>> {
        if self.hole.is_some() {
            return None;
        }
        self.observation.as_ref()?.admitted()
    }
    #[must_use]
    pub fn expression(&self) -> Option<&'a Expression<'a>> {
        self.admitted_expression()
            .map(|admitted| admitted.expression())
    }
    #[must_use]
    pub const fn hole(&self) -> Option<EmbedHole> {
        self.hole
    }

    pub fn comments(&self) -> impl Iterator<Item = CommentView<'_, 'a>> {
        self.observation
            .iter()
            .flat_map(|owner| owner.comments().iter())
            .map(|comment| CommentView {
                comment,
                coordinates: self.coordinates,
            })
    }
    pub fn diagnostics(&self) -> impl Iterator<Item = DiagnosticView<'_, 'a>> {
        self.diagnostics
            .iter()
            .chain(
                self.observation
                    .iter()
                    .flat_map(|owner| owner.diagnostics().iter()),
            )
            .map(|diagnostic| DiagnosticView {
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
    mut syntax: NativeSyntax<'a>,
) -> Result<RetainedExpression<'a>, Box<NativeSyntax<'a>>> {
    if syntax.grammar.shape != Shape::Expr {
        return Err(Box::new(syntax));
    }
    let observation = if let Some(owner) = syntax.embedding.take() {
        match owner.into_expression() {
            Ok(owner) => Some(owner),
            Err(owner) => {
                syntax.embedding = Some(*owner);
                return Err(Box::new(syntax));
            }
        }
    } else {
        None
    };
    let NativeSyntax {
        grammar,
        source_type,
        coordinates,
        observation: _,
        embedding: _,
        diagnostics,
        hole,
    } = syntax;
    Ok(RetainedExpression {
        grammar,
        source_type,
        coordinates,
        observation,
        diagnostics,
        hole,
    })
}

#[cfg(test)]
mod tests;
