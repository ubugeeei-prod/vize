//! Whole original For ownership for the bounded native semantic receiver.

use alloc::boxed::Box;
use oxc_ast::ast::{Expression, FormalParameter};
use vize_l0::Span;
use vize_l1::embed::syntax::{NativeForHead, NativeForRefusal};
use vize_l1::embed::{EmbedSource, SourceError};

use crate::resolution::source::ForReferenceSource;

/// Refusal preserves the entire original head and both available observations.
#[derive(Debug)]
pub struct RejectedForHeadInput<'a> {
    syntax: NativeForHead<'a>,
    pub kind: NativeForRefusal,
    pub span: Span,
}

impl<'a> RejectedForHeadInput<'a> {
    #[must_use]
    pub const fn syntax(&self) -> &NativeForHead<'a> {
        &self.syntax
    }
    #[must_use]
    pub fn into_syntax(self) -> NativeForHead<'a> {
        self.syntax
    }
}

/// Complete original Joint For input. A public source root may prove a local
/// physical value; this does not establish the complete selected Attribute.
/// Collection resolution, alias scopes, File/body/control and emission remain
/// the owning semantic receiver's obligations.
///
/// Caller-selected AST/source tuples cannot establish this input:
/// ```compile_fail
/// use oxc_ast::ast::{Expression, FormalParameter};
/// use vize_l1::embed::EmbedSource;
/// use vize_l2::lang::js::ForHeadInput;
/// fn substitute<'a>(aliases: &'a [FormalParameter<'a>],
///     collection: &'a Expression<'a>, source: EmbedSource<'a>) {
///     let _ = ForHeadInput::new(aliases, collection, source);
/// }
/// ```
/// The normal owner cannot be duplicated:
/// ```compile_fail
/// use vize_l2::lang::js::ForHeadInput;
/// fn duplicate(input: ForHeadInput<'_>) { let _ = input.clone(); }
/// ```
#[derive(Debug)]
pub struct ForHeadInput<'a> {
    syntax: NativeForHead<'a>,
    aliases: &'a [FormalParameter<'a>],
    collection: &'a Expression<'a>,
}

impl<'a> ForHeadInput<'a> {
    pub fn new(syntax: NativeForHead<'a>) -> Result<Self, Box<RejectedForHeadInput<'a>>> {
        let roots = ForReferenceSource::checked(&syntax)
            .map(|source| (source.aliases(), source.collection()));
        let Some((aliases, collection)) = roots else {
            return Err(Box::new(RejectedForHeadInput {
                kind: syntax
                    .native_refusal()
                    .unwrap_or(NativeForRefusal::StockProfile),
                span: syntax.source().span(),
                syntax,
            }));
        };
        Ok(Self {
            syntax,
            aliases,
            collection,
        })
    }

    #[must_use]
    pub const fn syntax(&self) -> &NativeForHead<'a> {
        &self.syntax
    }
    #[must_use]
    pub fn source(&self) -> EmbedSource<'a> {
        self.syntax.source()
    }
    /// Bare original arena roots grant neither binding identities nor control authority.
    #[must_use]
    pub const fn aliases(&self) -> &'a [FormalParameter<'a>] {
        self.aliases
    }
    #[must_use]
    pub const fn collection(&self) -> &'a Expression<'a> {
        self.collection
    }

    /// Decoded-relative alias coordinates, distinct from collection coordinates.
    pub fn alias_decoded_span(&self, span: oxc_span::Span) -> Result<Span, SourceError> {
        self.references()
            .ok_or(SourceError::InvalidDecodedSpan)?
            .alias_decoded_span(span)
    }
    pub fn alias_authored_span(&self, span: oxc_span::Span) -> Result<Span, SourceError> {
        self.references()
            .ok_or(SourceError::InvalidDecodedSpan)?
            .alias_authored_span(span)
    }
    /// Numeric projection checks coordinates, not descendant membership.
    pub fn collection_decoded_span(&self, span: oxc_span::Span) -> Result<Span, SourceError> {
        self.references()
            .ok_or(SourceError::InvalidDecodedSpan)?
            .collection_decoded_span(span)
    }
    pub fn collection_authored_span(&self, span: oxc_span::Span) -> Result<Span, SourceError> {
        self.references()
            .ok_or(SourceError::InvalidDecodedSpan)?
            .collection_authored_span(span)
    }
    #[must_use]
    pub fn into_syntax(self) -> NativeForHead<'a> {
        self.syntax
    }
    pub(crate) fn references(&self) -> Option<ForReferenceSource<'_, 'a>> {
        ForReferenceSource::checked(&self.syntax)
    }
}

mod native;
pub use native::{NativeForInput, RejectedNativeForInput};

#[cfg(test)]
mod tests;
