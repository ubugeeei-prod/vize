//! Consuming handoff of authored formal bindings, without generated containers.

use alloc::boxed::Box;
use oxc_ast::ast::{Comment, Expression, FormalParameter, FormalParameterRest, Statement};
use oxc_diagnostics::Diagnostics;
use oxc_span::SourceType;
use vize_l0::{Allocator, Span};

use super::{
    CommentView, DiagnosticView, EmbedHole, EmbedSource, Grammar, NativeSyntax, Shape, SourceError,
    coordinates::Coordinates,
};

/// Original authored parameter roots and owned observations from one parse.
///
/// The parameter slice remains at its original arena address. A rest root is
/// moved into the shared arena with all original descendants. Returned roots
/// can outlive this observation owner; diagnostics still drop normally. A local
/// hole exposes no recovery roots or generated parameter-list/arrow containers.
pub struct RetainedSlotParams<'a> {
    grammar: Grammar,
    source_type: SourceType,
    coordinates: Coordinates<'a>,
    parameters: Option<&'a [FormalParameter<'a>]>,
    rest: Option<&'a FormalParameterRest<'a>>,
    comments: &'a [Comment],
    diagnostics: Diagnostics,
    hole: Option<EmbedHole>,
}

impl core::fmt::Debug for RetainedSlotParams<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RetainedSlotParams")
            .field("grammar", &self.grammar)
            .field("source", &self.source())
            .field("hole", &self.hole)
            .field("diagnostic_count", &self.diagnostics.len())
            .finish_non_exhaustive()
    }
}

impl<'a> RetainedSlotParams<'a> {
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

    /// Actual generated prefix bytes in the original parser coordinates.
    #[must_use]
    pub const fn parser_prefix(&self) -> u32 {
        self.coordinates.prefix
    }

    #[must_use]
    pub const fn parameters(&self) -> Option<&'a [FormalParameter<'a>]> {
        self.parameters
    }

    #[must_use]
    pub const fn rest(&self) -> Option<&'a FormalParameterRest<'a>> {
        self.rest
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

pub(super) fn into_slot_params<'a>(
    syntax: NativeSyntax<'a>,
    allocator: &'a Allocator,
) -> Result<RetainedSlotParams<'a>, Box<NativeSyntax<'a>>> {
    if syntax.grammar.shape != Shape::SlotParams {
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
    let (mut parameters, mut rest) = (None, None);
    let mut comments = &[][..];
    if let Some(mut program) = program {
        comments = program.comments.into_arena_slice();
        if hole.is_none()
            && program.body.len() == 1
            && let Some(Statement::ExpressionStatement(statement)) = program.body.pop()
            && let Expression::ArrowFunctionExpression(arrow) = statement.unbox().expression
        {
            let params = arrow.unbox().params.unbox();
            parameters = Some(params.items.into_arena_slice());
            rest = params.rest.map(|root| &*allocator.alloc(root.unbox()));
        }
    }
    if hole.is_none() && parameters.is_none() {
        hole = Some(EmbedHole::InvalidWrappedShape);
    }
    Ok(RetainedSlotParams {
        grammar,
        source_type,
        coordinates,
        parameters,
        rest,
        comments,
        diagnostics,
        hole,
    })
}

#[cfg(test)]
mod tests;
