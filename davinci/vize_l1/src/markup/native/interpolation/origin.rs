use super::{
    NativeChild, NativeInterpolationError, NativeTemplateComponent, NativeTemplateGrammar,
};
use crate::{Element, Interpolation, SurfaceChild};
use vize_l0::{SourceBlock, Span};

/// Addresses identify immutable original arena backing and are compared only.
/// No selected/component wrapper address or borrow is retained across a move.
pub(super) struct Origin<'a> {
    pub(super) block: SourceBlock<'a>,
    template_index: usize,
    grammar: NativeTemplateGrammar,
    interpolation: *const Interpolation<'a>,
    parent: Option<*const Element<'a>>,
    ordinal: usize,
    pub(super) raw_content: &'a str,
    pub(super) content_span: Span,
    pub(super) full_span: Span,
}
impl<'a> Origin<'a> {
    pub(super) fn from_child(
        selected: &NativeTemplateComponent<'a>,
        child: &NativeChild<'_, 'a>,
        interpolation: &Interpolation<'a>,
    ) -> Result<Self, NativeInterpolationError> {
        if interpolation.open.is_missing()
            || interpolation.content.is_missing()
            || interpolation.close.is_missing()
            || interpolation.open.text != "{{"
            || interpolation.close.text != "}}"
            || interpolation.is_raw_html()
        {
            return Err(NativeInterpolationError::IncompleteInterpolation);
        }
        let block = selected.component().block();
        let span = |text| {
            block.span_of(text).ok_or(NativeInterpolationError::Source(
                crate::embed::SourceError::InvalidAuthoredSpan,
            ))
        };
        let open = span(interpolation.open.text)?;
        let content_span = span(interpolation.content.text)?;
        let close = span(interpolation.close.text)?;
        if open.end != content_span.start || content_span.end != close.start {
            return Err(NativeInterpolationError::IncompleteInterpolation);
        }
        Ok(Self {
            block,
            template_index: selected.template_index(),
            grammar: selected.grammar(),
            interpolation,
            parent: child.parent_element().map(core::ptr::from_ref),
            ordinal: child.ordinal(),
            raw_content: interpolation.content.text,
            content_span,
            full_span: Span::new(open.start, close.end),
        })
    }
    pub(super) fn matches(
        &self,
        selected: &NativeTemplateComponent<'a>,
        child: &NativeChild<'_, 'a>,
    ) -> bool {
        let SurfaceChild::Interpolation(interpolation) = child.surface() else {
            return false;
        };
        let block = selected.component().block();
        core::ptr::eq(child.component(), selected.component())
            && core::ptr::eq(self.interpolation, &**interpolation)
            && self.parent == child.parent_element().map(core::ptr::from_ref)
            && self.ordinal == child.ordinal()
            && self.template_index == selected.template_index()
            && self.grammar == selected.grammar()
            && self.block.start() == block.start()
            && core::ptr::eq(self.block.root_source(), block.root_source())
            && core::ptr::eq(self.block.source(), block.source())
            && core::ptr::eq(self.raw_content, interpolation.content.text)
            && block.span_of(interpolation.content.text) == Some(self.content_span)
            && block
                .span_of(interpolation.open.text)
                .is_some_and(|span| span.start == self.full_span.start)
            && block
                .span_of(interpolation.close.text)
                .is_some_and(|span| span.end == self.full_span.end)
    }
}
