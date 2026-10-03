//! Distinct original alias and collection coordinates borrowed from one Joint owner.

use oxc_ast::ast::{Expression, FormalParameter};
use vize_l0::Span;
use vize_l1::embed::{
    SourceError,
    syntax::{AdmittedDenseForHead, NativeForHead, RetainedExpression, RetainedSlotParams},
};

use super::ReferenceSource;

/// Only an original two-observation capability may establish this source.
/// This remains a short whole-owner borrow, never a public AST/source tuple.
pub(crate) struct ForReferenceSource<'h, 'a> {
    head: AdmittedDenseForHead<'h, 'a>,
    aliases: &'h RetainedSlotParams<'a>,
    collection: &'h RetainedExpression<'a>,
}

impl<'h, 'a> ForReferenceSource<'h, 'a> {
    pub(crate) fn checked(owner: &'h NativeForHead<'a>) -> Option<Self> {
        Some(Self {
            head: owner.admitted_dense()?,
            aliases: owner.aliases()?.ok()?,
            collection: owner.collection()?.ok()?,
        })
    }
    pub(crate) fn aliases(&self) -> &'a [FormalParameter<'a>] {
        self.head.aliases().parameters().items.as_slice()
    }
    pub(crate) fn collection(&self) -> &'a Expression<'a> {
        self.head.collection().expression()
    }
    /// Mint the collection namespace only from this complete original head.
    pub(in crate::resolution) fn collection_source(&self) -> ReferenceSource<'a> {
        ReferenceSource::ForCollection(self.collection.source().text())
    }
    pub(crate) fn alias_decoded_span(&self, span: oxc_span::Span) -> Result<Span, SourceError> {
        self.aliases.decoded_span(span)
    }
    pub(crate) fn alias_authored_span(&self, span: oxc_span::Span) -> Result<Span, SourceError> {
        self.aliases.authored_span(span)
    }
    pub(crate) fn collection_decoded_span(
        &self,
        span: oxc_span::Span,
    ) -> Result<Span, SourceError> {
        self.collection.decoded_span(span)
    }
    pub(crate) fn collection_authored_span(
        &self,
        span: oxc_span::Span,
    ) -> Result<Span, SourceError> {
        self.collection.authored_span(span)
    }
}
