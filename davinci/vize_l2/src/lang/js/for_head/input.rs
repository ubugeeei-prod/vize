//! Complete selected Attribute custody beside the original two For observations.

use alloc::boxed::Box;
use oxc_ast::ast::{Expression, FormalParameter};
use vize_l0::Span;
use vize_l1::embed::{SourceError, syntax::ForRefusal};
use vize_l1::markup::{
    AttributeForHead, AttributeForHeadView, AttributeView, NativeTemplateComponent,
};

use super::ForReferenceSource;

pub struct RejectedFileForInput<'a> {
    operand: AttributeForHead<'a>,
    pub kind: ForRefusal,
    pub span: Span,
}

impl<'a> RejectedFileForInput<'a> {
    #[must_use]
    pub const fn operand(&self) -> &AttributeForHead<'a> {
        &self.operand
    }
    #[must_use]
    pub fn into_operand(self) -> AttributeForHead<'a> {
        self.operand
    }
}

impl core::fmt::Debug for RejectedFileForInput<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RejectedFileForInput")
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
/// use vize_l2::lang::js::FileForInput;
/// fn duplicate(input: FileForInput<'_>) { let _ = input.clone(); }
/// ```
/// ```compile_fail
/// use vize_l1::markup::{AttributeView, AttributeForHeadView, NativeTemplateComponent};
/// use vize_l2::lang::js::FileForInput;
/// fn escape<'h, 'a>(input: FileForInput<'a>,
///     selected: &'h NativeTemplateComponent<'a>, attribute: AttributeView<'h, 'a>)
///     -> AttributeForHeadView<'h, 'a> {
///     input.admitted_for(selected, attribute).unwrap()
/// }
/// ```
pub struct FileForInput<'a> {
    operand: AttributeForHead<'a>,
    aliases: &'a [FormalParameter<'a>],
    collection: &'a Expression<'a>,
}

impl<'a> FileForInput<'a> {
    pub fn new(operand: AttributeForHead<'a>) -> Result<Self, Box<RejectedFileForInput<'a>>> {
        let roots = ForReferenceSource::checked(operand.syntax())
            .map(|source| (source.aliases(), source.collection()));
        let Some((aliases, collection)) = roots else {
            return Err(Box::new(RejectedFileForInput {
                kind: operand
                    .syntax()
                    .for_refusal()
                    .unwrap_or(ForRefusal::StockProfile),
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
    pub const fn operand(&self) -> &AttributeForHead<'a> {
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
        attribute: AttributeView<'h, 'a>,
    ) -> Option<AttributeForHeadView<'h, 'a>> {
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
    pub fn into_operand(self) -> AttributeForHead<'a> {
        self.operand
    }
    pub(crate) fn references(&self) -> Option<ForReferenceSource<'_, 'a>> {
        ForReferenceSource::checked(self.operand.syntax())
    }
}

impl core::fmt::Debug for FileForInput<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FileForInput")
            .field("syntax", self.operand.syntax())
            .finish_non_exhaustive()
    }
}
