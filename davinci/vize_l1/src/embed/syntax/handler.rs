//! Consuming the original parser-owned handler bodies and normal observations.

use alloc::boxed::Box;
use oxc_ast::ast::FunctionBody;
use oxc_diagnostics::Diagnostics;
use oxc_parser::{AdmittedHandlerBody, HandlerBodyObservation};
use oxc_span::SourceType;
use vize_l0::Span;

use super::{
    CommentView, DiagnosticView, EmbedHole, EmbedSource, Grammar, NativeSyntax, Shape, SourceError,
    coordinates::Coordinates,
};

/// Original arena body authenticated by its complete ordinary parser owner.
/// Raw arena roots can outlive this owner; admission remains a short owner borrow.
///
/// ```compile_fail
/// use vize_l1::embed::syntax::RetainedHandlerBody;
/// fn substitute<'a>(owner: &mut RetainedHandlerBody<'a>) {
///     owner.observation = None;
/// }
/// ```
pub struct RetainedHandlerBody<'a> {
    grammar: Grammar,
    source_type: SourceType,
    coordinates: Coordinates<'a>,
    observation: Option<HandlerBodyObservation<'a>>,
    diagnostics: Diagnostics,
    hole: Option<EmbedHole>,
}

impl core::fmt::Debug for RetainedHandlerBody<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RetainedHandlerBody")
            .field("grammar", &self.grammar)
            .field("source_type", &self.source_type)
            .field("source", &self.source())
            .field("hole", &self.hole)
            .field("diagnostic_count", &self.diagnostics().count())
            .finish_non_exhaustive()
    }
}

impl<'a> RetainedHandlerBody<'a> {
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
    /// Original stock admission borrowed from this owner. It does not establish
    /// event directive provenance, file association or identifier resolution.
    pub fn admitted_body(&self) -> Option<AdmittedHandlerBody<'_, 'a>> {
        if self.hole.is_some() {
            return None;
        }
        self.observation.as_ref()?.admitted()
    }

    /// The generated FunctionBody container is not an authored source range;
    /// only its directives and statements may be projected to authored spans.
    #[must_use]
    pub fn body(&self) -> Option<&'a FunctionBody<'a>> {
        self.admitted_body().map(|admitted| admitted.body())
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

pub(super) fn into_handler_body<'a>(
    mut syntax: NativeSyntax<'a>,
) -> Result<RetainedHandlerBody<'a>, Box<NativeSyntax<'a>>> {
    if syntax.grammar.shape != Shape::HandlerBody {
        return Err(Box::new(syntax));
    }
    let observation = if let Some(owner) = syntax.embedding.take() {
        match owner.into_handler_body() {
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
    let hole = hole.or_else(|| {
        observation
            .as_ref()
            .and_then(|owner| owner.hole())
            .map(super::wrapped::hole)
    });
    Ok(RetainedHandlerBody {
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
