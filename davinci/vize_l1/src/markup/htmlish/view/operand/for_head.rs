//! Whole once-retained Vue For observations at their genuine selected header.

use alloc::boxed::Box;
use vize_l0::Span;

use super::super::{AttributeView, SelectedTemplateComponent, TemplateGrammar};
use super::{AttributeOperandError, origin::Origin};
use crate::dialect::vue3::operand::for_head;
use crate::embed::syntax::{
    AdmittedDenseForHead, ForHead, ForInput, ForInputError, RejectedForInput,
};
use crate::embed::{Lang, SourceError};

/// Preparation failures keep any original rejected physical input intact.
#[derive(Debug)]
pub struct AttributeForHeadFailure<'a> {
    kind: AttributeOperandError,
    input: Option<Box<RejectedForInput<'a>>>,
}
impl AttributeForHeadFailure<'_> {
    #[must_use]
    pub const fn kind(&self) -> AttributeOperandError {
        self.kind
    }
    #[must_use]
    pub fn input(&self) -> Option<&RejectedForInput<'_>> {
        self.input.as_deref()
    }
}
impl<'a> From<AttributeOperandError> for AttributeForHeadFailure<'a> {
    fn from(kind: AttributeOperandError) -> Self {
        Self { kind, input: None }
    }
}

/// Complete original v-for value, both stock owners and private header origin.
/// No caller value, source range, language, allocator or raw AST can assemble it.
///
/// ```compile_fail
/// use vize_l1::markup::AttributeForHead;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<AttributeForHead<'static>>();
/// ```
///
/// ```compile_fail,E0616
/// use vize_l1::embed::syntax::ForHead;
/// use vize_l1::markup::AttributeForHead;
/// fn substitute<'a>(owner: &mut AttributeForHead<'a>, head: ForHead<'a>) {
///     owner.syntax = head;
/// }
/// ```
pub struct AttributeForHead<'a> {
    origin: Origin<'a>,
    syntax: ForHead<'a>,
}

impl<'a> SelectedTemplateComponent<'a> {
    /// Replace this actual header visit's value preparation and For parses.
    /// Full original value and Module JS/TS come from the same selected owner.
    /// Local refusal keeps the whole head and every available stock observation.
    pub fn observe_attribute_for_head(
        &self,
        attribute: AttributeView<'_, 'a>,
    ) -> Result<AttributeForHead<'a>, AttributeForHeadFailure<'a>> {
        Origin::check_original_header(self, &attribute)?;
        let block = self.component().block();
        let name_offset =
            block
                .offset_of(attribute.surface().name.text)
                .ok_or(AttributeOperandError::Source(
                    SourceError::InvalidAuthoredSpan,
                ))?;
        if !for_head(
            attribute.surface().name.text,
            name_offset,
            block.root_source(),
        )
        .map_err(AttributeOperandError::Directive)?
        {
            return Err(AttributeOperandError::UnsupportedDirective.into());
        }
        let origin = Origin::from_attribute(self, &attribute)?;
        let lang = match self.grammar() {
            TemplateGrammar::JavaScriptModule => Lang::Js,
            TemplateGrammar::TypeScriptModule => Lang::Ts,
        };
        let input = ForInput::attribute_in(
            self.component().allocator(),
            origin.block,
            origin.raw_value,
            lang,
        )
        .map_err(|input| AttributeForHeadFailure {
            kind: AttributeOperandError::Source(match input.error() {
                ForInputError::ForeignValue => SourceError::InvalidAuthoredSpan,
                ForInputError::Source(error) => error,
            }),
            input: Some(input),
        })?;
        Ok(AttributeForHead {
            origin,
            syntax: input.observe(),
        })
    }
}

impl<'a> AttributeForHead<'a> {
    /// Readonly complete observations, including local typed syntax/family refusals.
    #[must_use]
    pub fn syntax(&self) -> &ForHead<'a> {
        &self.syntax
    }
    #[must_use]
    pub const fn raw_value(&self) -> &'a str {
        self.origin.raw_value
    }
    #[must_use]
    pub const fn value_span(&self) -> Span {
        self.origin.value_span
    }
    #[must_use]
    pub const fn name_span(&self) -> Span {
        self.origin.name_span
    }
    /// Rejoin the complete original Attribute after owner movement/storage growth.
    pub fn admitted_for<'s>(
        &'s self,
        selected: &'s SelectedTemplateComponent<'a>,
        attribute: AttributeView<'s, 'a>,
    ) -> Option<AttributeForHeadView<'s, 'a>> {
        if !self.origin.matches(selected, &attribute) {
            return None;
        }
        self.syntax.admitted_dense()?;
        Some(AttributeForHeadView {
            owner: self,
            selected,
            attribute,
        })
    }
    /// Moving out the original whole head deliberately discards Attribute authority.
    #[must_use]
    pub fn into_syntax(self) -> ForHead<'a> {
        self.syntax
    }
}

/// Short actual header/For join, without File/body, scope or execution completion.
///
/// ```compile_fail,E0515
/// use vize_l1::markup::{AttributeView, AttributeForHead,
///     AttributeForHeadView, SelectedTemplateComponent};
/// fn escape<'s, 'a>(owner: AttributeForHead<'a>,
///     selected: &'s SelectedTemplateComponent<'a>, attribute: AttributeView<'s, 'a>)
///     -> AttributeForHeadView<'s, 'a> {
///     owner.admitted_for(selected, attribute).unwrap()
/// }
/// ```
///
/// ```compile_fail,E0515
/// use vize_l1::markup::{AttributeView, AttributeForHead,
///     AttributeForHeadView, SelectedTemplateComponent};
/// fn escape_selected<'s, 'a>(owner: &'s AttributeForHead<'a>,
///     selected: SelectedTemplateComponent<'a>, attribute: AttributeView<'s, 'a>)
///     -> AttributeForHeadView<'s, 'a> {
///     owner.admitted_for(&selected, attribute).unwrap()
/// }
/// ```
pub struct AttributeForHeadView<'s, 'a> {
    owner: &'s AttributeForHead<'a>,
    selected: &'s SelectedTemplateComponent<'a>,
    attribute: AttributeView<'s, 'a>,
}
impl<'s, 'a> AttributeForHeadView<'s, 'a> {
    #[must_use]
    pub fn selected(&self) -> &'s SelectedTemplateComponent<'a> {
        self.selected
    }
    #[must_use]
    pub fn attribute(&self) -> &AttributeView<'s, 'a> {
        &self.attribute
    }
    #[must_use]
    pub fn operand(&self) -> &'s AttributeForHead<'a> {
        self.owner
    }
    /// Both capabilities borrow the same original retained whole-head owner.
    #[must_use]
    pub fn for_head(&self) -> Option<AdmittedDenseForHead<'s, 'a>> {
        self.owner.syntax.admitted_dense()
    }
}

#[cfg(test)]
mod tests;
