use oxc_ast::ast::{Expression, FormalParameter};
use vize_l0::Span;

use super::{Coordinates, EmbedSource, SourceError};

/// Real dense binding/collection roots plus independent checked projections.
/// Roots and coordinate/source copies can outlive the observation owner.
#[derive(Debug, Clone, Copy)]
pub struct DenseForHeadView<'a> {
    pub(super) parameters: &'a [FormalParameter<'a>],
    pub(super) collection: &'a Expression<'a>,
    pub(super) alias_coordinates: Coordinates<'a>,
    pub(super) collection_coordinates: Coordinates<'a>,
}

impl<'a> DenseForHeadView<'a> {
    #[must_use]
    pub const fn parameters(self) -> &'a [FormalParameter<'a>] {
        self.parameters
    }
    #[must_use]
    pub const fn collection(self) -> &'a Expression<'a> {
        self.collection
    }
    #[must_use]
    pub const fn alias_source(self) -> EmbedSource<'a> {
        self.alias_coordinates.source
    }
    #[must_use]
    pub const fn collection_source(self) -> EmbedSource<'a> {
        self.collection_coordinates.source
    }
    #[must_use]
    pub const fn alias_prefix(self) -> u32 {
        self.alias_coordinates.prefix
    }
    #[must_use]
    pub const fn collection_prefix(self) -> u32 {
        self.collection_coordinates.prefix
    }
    pub fn alias_decoded_span(self, span: oxc_span::Span) -> Result<Span, SourceError> {
        self.alias_coordinates.decoded_span(span)
    }
    pub fn alias_authored_span(self, span: oxc_span::Span) -> Result<Span, SourceError> {
        self.alias_source()
            .authored_span(self.alias_decoded_span(span)?)
    }
    pub fn collection_decoded_span(self, span: oxc_span::Span) -> Result<Span, SourceError> {
        self.collection_coordinates.decoded_span(span)
    }
    pub fn collection_authored_span(self, span: oxc_span::Span) -> Result<Span, SourceError> {
        self.collection_source()
            .authored_span(self.collection_decoded_span(span)?)
    }
}
