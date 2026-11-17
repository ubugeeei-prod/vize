//! A valueless original Else header is an owner, never an empty expression.

use super::{
    Attribute, AttributeHead, AttributeOperandError, AttributeView, Element, Origin,
    TemplateComponent, TemplateGrammar,
};
use crate::markup::{DirectivePrefix, ElementClose};
use vize_l0::{SourceBlock, Span};

/// The actual argument/modifier/value-free Vue Else attribute and its extent.
/// It grants no sibling, body, File or target completion authority.
///
/// ```compile_fail
/// use vize_l1::markup::AttributeElse;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<AttributeElse<'static>>();
/// ```
pub struct AttributeElse<'a> {
    block: SourceBlock<'a>,
    template_index: usize,
    grammar: TemplateGrammar,
    element: *const Element<'a>,
    attribute: *const Attribute<'a>,
    ordinal: usize,
    name: &'a str,
    name_span: Span,
    element_span: Span,
}

impl<'a> AttributeHead<'_, 'a> {
    /// Reuse this sealed head's original membership and dialect decomposition.
    /// This path neither prepares a value nor calls an expression parser.
    pub fn observe_else(&self) -> Result<AttributeElse<'a>, AttributeOperandError> {
        let selected = self.selected();
        let attribute = self.attribute();
        let surface = attribute.surface();
        let Some(directive) = self.directive() else {
            return Err(AttributeOperandError::UnsupportedDirective);
        };
        if directive.prefix != DirectivePrefix::Full
            || directive.arg.is_some()
            || directive.modifiers.start != directive.modifiers.end
            || directive.name.end != self.name_block().end()
            || directive.name.slice(self.name_block().root_source()) != "else"
            || surface.name.is_missing()
            || surface.eq.is_some()
            || surface.value.is_some()
        {
            return Err(AttributeOperandError::UnsupportedDirective);
        }
        let block = selected.component().block();
        let element = attribute.element();
        let opening =
            block
                .span_of(element.open.lt_name.text)
                .ok_or(AttributeOperandError::Source(
                    crate::embed::SourceError::InvalidAuthoredSpan,
                ))?;
        let ending = match &element.close {
            ElementClose::Present(close) => block.span_of(close.gt.text),
            ElementClose::NotExpected => block.span_of(element.open.gt.text),
            ElementClose::Missing | ElementClose::Implicit => None,
        }
        .ok_or(AttributeOperandError::UnexpectedShape)?;
        Ok(AttributeElse {
            block,
            template_index: selected.template_index(),
            grammar: selected.grammar(),
            element,
            attribute: surface,
            ordinal: attribute.ordinal(),
            name: surface.name.text,
            name_span: self.name_block().span(),
            element_span: Span::new(opening.start, ending.end),
        })
    }
}

impl<'a> AttributeElse<'a> {
    #[must_use]
    pub const fn name_span(&self) -> Span {
        self.name_span
    }
    /// Extent derived from this actual original Element's opening and ending.
    #[must_use]
    pub const fn element_span(&self) -> Span {
        self.element_span
    }
    #[must_use]
    pub const fn source_block(&self) -> SourceBlock<'a> {
        self.block
    }
    /// Reborrow the exact original attribute, not equal text or caller spans.
    pub fn admitted_for<'s>(
        &'s self,
        selected: &'s TemplateComponent<'a>,
        attribute: AttributeView<'s, 'a>,
    ) -> Option<AttributeElseView<'s, 'a>> {
        let block = selected.component().block();
        let surface = attribute.surface();
        if Origin::check_original_header(selected, &attribute).is_err()
            || !core::ptr::eq(self.element, attribute.element())
            || !core::ptr::eq(self.attribute, surface)
            || self.ordinal != attribute.ordinal()
            || self.template_index != selected.template_index()
            || self.grammar != selected.grammar()
            || self.block.start() != block.start()
            || !core::ptr::eq(self.block.root_source(), block.root_source())
            || !core::ptr::eq(self.block.source(), block.source())
            || !core::ptr::eq(self.name, surface.name.text)
            || block.span_of(surface.name.text) != Some(self.name_span)
            || surface.eq.is_some()
            || surface.value.is_some()
        {
            return None;
        }
        Some(AttributeElseView {
            owner: self,
            selected,
            attribute,
        })
    }
}

/// A short original header join, without whole branch or sibling authority.
pub struct AttributeElseView<'s, 'a> {
    owner: &'s AttributeElse<'a>,
    selected: &'s TemplateComponent<'a>,
    attribute: AttributeView<'s, 'a>,
}
impl<'s, 'a> AttributeElseView<'s, 'a> {
    #[must_use]
    pub const fn owner(&self) -> &'s AttributeElse<'a> {
        self.owner
    }
    #[must_use]
    pub const fn selected(&self) -> &'s TemplateComponent<'a> {
        self.selected
    }
    #[must_use]
    pub fn attribute(&self) -> &AttributeView<'s, 'a> {
        &self.attribute
    }
}

#[cfg(test)]
mod tests;
