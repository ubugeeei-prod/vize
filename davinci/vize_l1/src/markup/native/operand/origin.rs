use super::{
    Attribute, Element, NativeAttribute, NativeAttributeOperandError, NativeTemplateComponent,
    NativeTemplateGrammar, SourceBlock, SourceError, Span,
};
use crate::ElementClose;

pub(super) struct Origin<'a> {
    pub(super) block: SourceBlock<'a>,
    template_index: usize,
    grammar: NativeTemplateGrammar,
    element: *const Element<'a>,
    attribute: *const Attribute<'a>,
    ordinal: usize,
    raw_name: &'a str,
    pub(super) name_span: Span,
    pub(super) raw_value: &'a str,
    pub(super) value_span: Span,
}
impl<'a> Origin<'a> {
    /// Preserve original membership/recovery/verbatim precedence before dialect selection.
    pub(super) fn check_original_header(
        selected: &NativeTemplateComponent<'a>,
        attribute: &NativeAttribute<'_, 'a>,
    ) -> Result<(), NativeAttributeOperandError> {
        if !core::ptr::eq(attribute.component(), selected.component()) {
            return Err(NativeAttributeOperandError::ForeignComponent);
        }
        let carrier = selected.component().carrier();
        if !carrier.errors.is_empty() || !carrier.unsupported.is_empty() {
            return Err(NativeAttributeOperandError::RecoveredComponent);
        }
        // Node/token holes are retained on the original surface even when the
        // parser has no diagnostic row. Inspect only this actual header/extent.
        let element = attribute.element();
        if element.open.lt_name.is_missing()
            || element.open.gt.is_missing()
            || element
                .open
                .slash
                .as_ref()
                .is_some_and(|slash| slash.is_missing())
            || match &element.close {
                ElementClose::Missing => true,
                ElementClose::Present(close) => {
                    close.lt_slash_name.is_missing() || close.gt.is_missing()
                }
                ElementClose::Implicit | ElementClose::NotExpected => false,
            }
        {
            return Err(NativeAttributeOperandError::RecoveredComponent);
        }
        if attribute.element().open.is_verbatim() {
            return Err(NativeAttributeOperandError::Verbatim);
        }
        Ok(())
    }

    pub(super) fn from_attribute(
        selected: &NativeTemplateComponent<'a>,
        attribute: &NativeAttribute<'_, 'a>,
    ) -> Result<Self, NativeAttributeOperandError> {
        let surface = attribute.surface();
        let value = surface
            .value
            .as_ref()
            .ok_or(NativeAttributeOperandError::IncompleteValue)?;
        if surface.name.is_missing()
            || surface.eq.as_ref().is_none_or(|eq| eq.is_missing())
            || value.content.is_missing()
            || value
                .open_quote
                .as_ref()
                .is_some_and(|quote| quote.is_missing())
            || value
                .close_quote
                .as_ref()
                .is_some_and(|quote| quote.is_missing())
            || value.open_quote.is_some() != value.close_quote.is_some()
        {
            return Err(NativeAttributeOperandError::IncompleteValue);
        }
        let block = selected.component().block();
        let name_span =
            block
                .span_of(surface.name.text)
                .ok_or(NativeAttributeOperandError::Source(
                    SourceError::InvalidAuthoredSpan,
                ))?;
        let value_span =
            block
                .span_of(value.content.text)
                .ok_or(NativeAttributeOperandError::Source(
                    SourceError::InvalidAuthoredSpan,
                ))?;
        Ok(Self {
            block,
            template_index: selected.template_index(),
            grammar: selected.grammar(),
            element: attribute.element(),
            attribute: surface,
            ordinal: attribute.ordinal(),
            raw_name: surface.name.text,
            name_span,
            raw_value: value.content.text,
            value_span,
        })
    }
    pub(super) fn matches(
        &self,
        selected: &NativeTemplateComponent<'a>,
        attribute: &NativeAttribute<'_, 'a>,
    ) -> bool {
        let block = selected.component().block();
        core::ptr::eq(attribute.component(), selected.component())
            && core::ptr::eq(self.element, attribute.element())
            && core::ptr::eq(self.attribute, attribute.surface())
            && self.ordinal == attribute.ordinal()
            && self.template_index == selected.template_index()
            && self.grammar == selected.grammar()
            && self.block.start() == block.start()
            && core::ptr::eq(self.block.root_source(), block.root_source())
            && core::ptr::eq(self.block.source(), block.source())
            && core::ptr::eq(self.raw_name, attribute.surface().name.text)
            && block.span_of(attribute.surface().name.text) == Some(self.name_span)
            && attribute.surface().value.as_ref().is_some_and(|value| {
                core::ptr::eq(self.raw_value, value.content.text)
                    && block.span_of(value.content.text) == Some(self.value_span)
            })
    }
}
