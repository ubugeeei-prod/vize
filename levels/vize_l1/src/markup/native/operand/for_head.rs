//! Whole once-retained Vue For observations at their genuine selected header.

use alloc::boxed::Box;
use vize_l0::Span;

use super::super::{NativeAttribute, NativeTemplateComponent, NativeTemplateGrammar};
use super::{NativeAttributeOperandError, origin::Origin};
use crate::dialect::vue3::operand::for_head;
use crate::embed::syntax::{
    AdmittedDenseForHead, NativeForHead, NativeForInput, NativeForInputError,
    RejectedNativeForInput,
};
use crate::embed::{Lang, SourceError};

/// Preparation failures keep any original rejected physical input intact.
#[derive(Debug)]
pub struct NativeAttributeForHeadFailure<'a> {
    kind: NativeAttributeOperandError,
    input: Option<Box<RejectedNativeForInput<'a>>>,
}
impl NativeAttributeForHeadFailure<'_> {
    #[must_use]
    pub const fn kind(&self) -> NativeAttributeOperandError {
        self.kind
    }
    #[must_use]
    pub fn input(&self) -> Option<&RejectedNativeForInput<'_>> {
        self.input.as_deref()
    }
}
impl<'a> From<NativeAttributeOperandError> for NativeAttributeForHeadFailure<'a> {
    fn from(kind: NativeAttributeOperandError) -> Self {
        Self { kind, input: None }
    }
}

/// Complete original v-for value, both stock owners and private header origin.
/// No caller value, source range, language, allocator or raw AST can assemble it.
///
/// ```compile_fail
/// use vize_l1::markup::NativeAttributeForHead;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<NativeAttributeForHead<'static>>();
/// ```
///
/// ```compile_fail,E0616
/// use vize_l1::embed::syntax::NativeForHead;
/// use vize_l1::markup::NativeAttributeForHead;
/// fn substitute<'a>(owner: &mut NativeAttributeForHead<'a>, head: NativeForHead<'a>) {
///     owner.syntax = head;
/// }
/// ```
pub struct NativeAttributeForHead<'a> {
    origin: Origin<'a>,
    syntax: NativeForHead<'a>,
}

impl<'a> NativeTemplateComponent<'a> {
    /// Replace this actual header visit's value preparation and For parses.
    /// Full original value and Module JS/TS come from the same selected owner.
    /// Local refusal keeps the whole head and every available stock observation.
    pub fn observe_attribute_for_head(
        &self,
        attribute: NativeAttribute<'_, 'a>,
    ) -> Result<NativeAttributeForHead<'a>, NativeAttributeForHeadFailure<'a>> {
        Origin::check_original_header(self, &attribute)?;
        let block = self.component().block();
        let name_offset = block.offset_of(attribute.surface().name.text).ok_or(
            NativeAttributeOperandError::Source(SourceError::InvalidAuthoredSpan),
        )?;
        if !for_head(
            attribute.surface().name.text,
            name_offset,
            block.root_source(),
        )
        .map_err(NativeAttributeOperandError::Directive)?
        {
            return Err(NativeAttributeOperandError::UnsupportedDirective.into());
        }
        let origin = Origin::from_attribute(self, &attribute)?;
        let lang = match self.grammar() {
            NativeTemplateGrammar::JavaScriptModule => Lang::Js,
            NativeTemplateGrammar::TypeScriptModule => Lang::Ts,
        };
        let input = NativeForInput::attribute_in(
            self.component().allocator(),
            origin.block,
            origin.raw_value,
            lang,
        )
        .map_err(|input| NativeAttributeForHeadFailure {
            kind: NativeAttributeOperandError::Source(match input.error() {
                NativeForInputError::ForeignValue => SourceError::InvalidAuthoredSpan,
                NativeForInputError::Source(error) => error,
            }),
            input: Some(input),
        })?;
        Ok(NativeAttributeForHead {
            origin,
            syntax: input.observe(),
        })
    }
}

impl<'a> NativeAttributeForHead<'a> {
    /// Readonly complete observations, including local typed syntax/family refusals.
    #[must_use]
    pub fn syntax(&self) -> &NativeForHead<'a> {
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
        selected: &'s NativeTemplateComponent<'a>,
        attribute: NativeAttribute<'s, 'a>,
    ) -> Option<NativeAttributeForHeadView<'s, 'a>> {
        if !self.origin.matches(selected, &attribute) {
            return None;
        }
        self.syntax.admitted_dense()?;
        Some(NativeAttributeForHeadView {
            owner: self,
            selected,
            attribute,
        })
    }
    /// Moving out the original whole head deliberately discards Attribute authority.
    #[must_use]
    pub fn into_syntax(self) -> NativeForHead<'a> {
        self.syntax
    }
}

/// Short actual header/For join, without File/body, scope or execution completion.
///
/// ```compile_fail,E0515
/// use vize_l1::markup::{NativeAttribute, NativeAttributeForHead,
///     NativeAttributeForHeadView, NativeTemplateComponent};
/// fn escape<'s, 'a>(owner: NativeAttributeForHead<'a>,
///     selected: &'s NativeTemplateComponent<'a>, attribute: NativeAttribute<'s, 'a>)
///     -> NativeAttributeForHeadView<'s, 'a> {
///     owner.admitted_for(selected, attribute).unwrap()
/// }
/// ```
///
/// ```compile_fail,E0515
/// use vize_l1::markup::{NativeAttribute, NativeAttributeForHead,
///     NativeAttributeForHeadView, NativeTemplateComponent};
/// fn escape_selected<'s, 'a>(owner: &'s NativeAttributeForHead<'a>,
///     selected: NativeTemplateComponent<'a>, attribute: NativeAttribute<'s, 'a>)
///     -> NativeAttributeForHeadView<'s, 'a> {
///     owner.admitted_for(&selected, attribute).unwrap()
/// }
/// ```
pub struct NativeAttributeForHeadView<'s, 'a> {
    owner: &'s NativeAttributeForHead<'a>,
    selected: &'s NativeTemplateComponent<'a>,
    attribute: NativeAttribute<'s, 'a>,
}
impl<'s, 'a> NativeAttributeForHeadView<'s, 'a> {
    #[must_use]
    pub fn selected(&self) -> &'s NativeTemplateComponent<'a> {
        self.selected
    }
    #[must_use]
    pub fn attribute(&self) -> &NativeAttribute<'s, 'a> {
        &self.attribute
    }
    #[must_use]
    pub fn operand(&self) -> &'s NativeAttributeForHead<'a> {
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
