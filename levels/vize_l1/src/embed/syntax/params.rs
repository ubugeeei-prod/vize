//! Consuming the original parser-owned formal bindings and normal observations.

use alloc::boxed::Box;
use oxc_ast::ast::{FormalParameter, FormalParameterRest};
use oxc_diagnostics::Diagnostics;
use oxc_parser::{AdmittedParameters, ParametersObservation};
use oxc_span::SourceType;
use vize_l0::Span;

use super::{
    CommentView, DiagnosticView, EmbedHole, EmbedSource, Grammar, NativeSyntax, Shape, SourceError,
    coordinates::Coordinates,
};

/// Original arena bindings authenticated by their complete ordinary parser owner.
/// Raw arena roots can outlive this owner; admission remains a short owner borrow.
pub struct RetainedSlotParams<'a> {
    grammar: Grammar,
    source_type: SourceType,
    coordinates: Coordinates<'a>,
    observation: Option<ParametersObservation<'a>>,
    diagnostics: Diagnostics,
    hole: Option<EmbedHole>,
}

impl core::fmt::Debug for RetainedSlotParams<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RetainedSlotParams")
            .field("grammar", &self.grammar)
            .field("source_type", &self.source_type)
            .field("source", &self.source())
            .field("hole", &self.hole)
            .field("diagnostic_count", &self.diagnostics().count())
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
    /// Original stock admission borrowed from this owner. It does not establish
    /// joint For-head provenance, file association or generic binding uniqueness.
    pub fn admitted_parameters(&self) -> Option<AdmittedParameters<'_, 'a>> {
        if self.hole.is_some() {
            return None;
        }
        self.observation.as_ref()?.admitted()
    }

    #[must_use]
    pub fn parameters(&self) -> Option<&'a [FormalParameter<'a>]> {
        self.admitted_parameters()
            .map(|admitted| admitted.parameters().items.as_slice())
    }

    #[must_use]
    pub fn rest(&self) -> Option<&'a FormalParameterRest<'a>> {
        self.admitted_parameters()
            .and_then(|admitted| admitted.parameters().rest.as_deref())
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

pub(super) fn into_slot_params<'a>(
    mut syntax: NativeSyntax<'a>,
) -> Result<RetainedSlotParams<'a>, Box<NativeSyntax<'a>>> {
    if syntax.grammar.shape != Shape::SlotParams {
        return Err(Box::new(syntax));
    }
    let observation = if let Some(owner) = syntax.embedding.take() {
        match owner.into_parameters() {
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
    Ok(RetainedSlotParams {
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

#[cfg(test)]
mod authority_tests;
