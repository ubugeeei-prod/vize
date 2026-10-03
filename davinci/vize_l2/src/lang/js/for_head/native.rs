//! Complete selected Attribute custody beside the original two For observations.

use alloc::boxed::Box;
use oxc_ast::ast::{Expression, FormalParameter};
use vize_l0::Span;
use vize_l1::embed::{SourceError, syntax::NativeForRefusal};
use vize_l1::markup::{
    NativeAttribute, NativeAttributeForHead, NativeAttributeForHeadView, NativeTemplateComponent,
};

use super::ForReferenceSource;

pub struct RejectedNativeForInput<'a> {
    operand: NativeAttributeForHead<'a>,
    pub kind: NativeForRefusal,
    pub span: Span,
}

impl<'a> RejectedNativeForInput<'a> {
    #[must_use]
    pub const fn operand(&self) -> &NativeAttributeForHead<'a> {
        &self.operand
    }
    #[must_use]
    pub fn into_operand(self) -> NativeAttributeForHead<'a> {
        self.operand
    }
}

impl core::fmt::Debug for RejectedNativeForInput<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RejectedNativeForInput")
            .field("kind", &self.kind)
            .field("span", &self.span)
            .field("syntax", self.operand.syntax())
            .finish_non_exhaustive()
    }
}

/// Movable genuine Attribute/For input. No caller AST, source tuple, language or
/// ordinary For input can manufacture its selected For association.
/// No File, collection resolution, child scope or canonical body is minted here.
///
/// ```compile_fail
/// use vize_l2::lang::js::NativeForInput;
/// fn duplicate(input: NativeForInput<'_>) { let _ = input.clone(); }
/// ```
/// ```compile_fail
/// use vize_l1::markup::{NativeAttribute, NativeAttributeForHeadView, NativeTemplateComponent};
/// use vize_l2::lang::js::NativeForInput;
/// fn escape<'h, 'a>(input: NativeForInput<'a>,
///     selected: &'h NativeTemplateComponent<'a>, attribute: NativeAttribute<'h, 'a>)
///     -> NativeAttributeForHeadView<'h, 'a> {
///     input.admitted_for(selected, attribute).unwrap()
/// }
/// ```
pub struct NativeForInput<'a> {
    operand: NativeAttributeForHead<'a>,
    aliases: &'a [FormalParameter<'a>],
    collection: &'a Expression<'a>,
}

impl<'a> NativeForInput<'a> {
    pub fn new(
        operand: NativeAttributeForHead<'a>,
    ) -> Result<Self, Box<RejectedNativeForInput<'a>>> {
        let roots = ForReferenceSource::checked(operand.syntax())
            .map(|source| (source.aliases(), source.collection()));
        let Some((aliases, collection)) = roots else {
            return Err(Box::new(RejectedNativeForInput {
                kind: operand
                    .syntax()
                    .native_refusal()
                    .unwrap_or(NativeForRefusal::StockProfile),
                span: operand.value_span(),
                operand,
            }));
        };
        Ok(Self {
            operand,
            aliases,
            collection,
        })
    }
    #[must_use]
    pub const fn operand(&self) -> &NativeAttributeForHead<'a> {
        &self.operand
    }
    #[must_use]
    pub const fn aliases(&self) -> &'a [FormalParameter<'a>] {
        self.aliases
    }
    #[must_use]
    pub const fn collection(&self) -> &'a Expression<'a> {
        self.collection
    }
    pub fn admitted_for<'h>(
        &'h self,
        selected: &'h NativeTemplateComponent<'a>,
        attribute: NativeAttribute<'h, 'a>,
    ) -> Option<NativeAttributeForHeadView<'h, 'a>> {
        self.operand.admitted_for(selected, attribute)
    }
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
    pub fn into_operand(self) -> NativeAttributeForHead<'a> {
        self.operand
    }
    pub(crate) fn references(&self) -> Option<ForReferenceSource<'_, 'a>> {
        ForReferenceSource::checked(self.operand.syntax())
    }
}

impl core::fmt::Debug for NativeForInput<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NativeForInput")
            .field("syntax", self.operand.syntax())
            .finish_non_exhaustive()
    }
}
