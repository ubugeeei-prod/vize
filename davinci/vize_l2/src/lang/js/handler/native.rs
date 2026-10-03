//! Original selected event ownership retained beside its whole handler body.

use alloc::boxed::Box;
use oxc_ast::ast::FunctionBody;
use vize_l0::Span;
use vize_l1::embed::SourceError;
use vize_l1::markup::{
    NativeAttribute, NativeAttributeHandler, NativeAttributeHandlerView, NativeTemplateComponent,
};

use super::{HandlerInputErrorKind, HandlerReferenceSource};

pub struct RejectedNativeHandlerInput<'a> {
    operand: NativeAttributeHandler<'a>,
    pub kind: HandlerInputErrorKind,
    pub span: Span,
}

impl<'a> RejectedNativeHandlerInput<'a> {
    #[must_use]
    pub const fn operand(&self) -> &NativeAttributeHandler<'a> {
        &self.operand
    }

    #[must_use]
    pub fn into_operand(self) -> NativeAttributeHandler<'a> {
        self.operand
    }
}

impl core::fmt::Debug for RejectedNativeHandlerInput<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RejectedNativeHandlerInput")
            .field("kind", &self.kind)
            .field("span", &self.span)
            .field("syntax", self.operand.syntax())
            .finish_non_exhaustive()
    }
}

/// Movable original event owner for a future authentic File header receiver.
/// No File, scopes, reference resolution or ui.on association is minted here.
///
/// ```compile_fail
/// use vize_l2::lang::js::NativeHandlerInput;
/// fn duplicate(input: NativeHandlerInput<'_>) {
///     let _ = input.clone();
/// }
/// ```
pub struct NativeHandlerInput<'a> {
    operand: NativeAttributeHandler<'a>,
    body: &'a FunctionBody<'a>,
}

impl<'a> NativeHandlerInput<'a> {
    pub fn new(
        operand: NativeAttributeHandler<'a>,
    ) -> Result<Self, Box<RejectedNativeHandlerInput<'a>>> {
        let Some(body) =
            HandlerReferenceSource::checked(operand.syntax()).map(|source| source.body())
        else {
            return Err(Box::new(RejectedNativeHandlerInput {
                kind: HandlerInputErrorKind::IncompleteSyntax,
                span: operand.value_span(),
                operand,
            }));
        };
        Ok(Self { operand, body })
    }

    #[must_use]
    pub const fn operand(&self) -> &NativeAttributeHandler<'a> {
        &self.operand
    }

    #[must_use]
    pub const fn body(&self) -> &'a FunctionBody<'a> {
        self.body
    }

    /// Rejoin only the actual original selected header; numeric projection and
    /// normally owned syntax cannot replace this event association.
    #[must_use]
    pub fn admitted_for<'h>(
        &'h self,
        selected: &'h NativeTemplateComponent<'a>,
        attribute: NativeAttribute<'h, 'a>,
    ) -> Option<NativeAttributeHandlerView<'h, 'a>> {
        self.operand.admitted_for(selected, attribute)
    }

    pub fn decoded_span(&self, span: oxc_span::Span) -> Result<Span, SourceError> {
        self.references().decoded_span(span)
    }

    pub fn authored_span(&self, span: oxc_span::Span) -> Result<Span, SourceError> {
        self.references().authored_span(span)
    }

    #[must_use]
    pub fn into_operand(self) -> NativeAttributeHandler<'a> {
        self.operand
    }

    pub(crate) fn references(&self) -> HandlerReferenceSource<'_, 'a> {
        HandlerReferenceSource::from_native(self)
    }
}

impl core::fmt::Debug for NativeHandlerInput<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NativeHandlerInput")
            .field("syntax", self.operand.syntax())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests;
