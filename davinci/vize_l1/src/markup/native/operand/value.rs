//! Original attribute-value preparation without expression or target admission.

use super::origin::Origin;
use super::{NativeAttribute, NativeAttributeOperandError, NativeTemplateComponent};
use crate::embed::{EmbedSource, prepare_attribute_value};
use vize_l0::Span;

mod failure;
mod frame;
pub use failure::NativeAttributeValueFailure;
use frame::Frame;

/// A normally owned receipt from one original attribute preparation event.
/// Its source and complete decode map are kept unchanged. Neither caller source
/// coordinates nor an ordinary `EmbedSource` can manufacture this receipt.
/// It does not classify a directive, validate a target name or complete a File.
///
/// ```compile_fail
/// use vize_l1::markup::NativeAttributeValue;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<NativeAttributeValue<'static>>();
/// ```
///
/// ```compile_fail
/// use vize_l1::{embed::EmbedSource, markup::NativeAttributeValue};
/// fn fabricated(source: EmbedSource<'static>) -> NativeAttributeValue<'static> {
///     NativeAttributeValue { source }
/// }
/// ```
pub struct NativeAttributeValue<'a> {
    origin: Origin<'a>,
    frame: Frame,
    source: EmbedSource<'a>,
}

impl<'a> NativeTemplateComponent<'a> {
    /// Prepare the complete value during this actual opening-header visit.
    /// Original membership, recovery and complete token geometry are checked
    /// before the existing attribute-context decoder is called exactly once.
    /// No JavaScript parse or target-specific name policy is involved.
    pub fn observe_attribute_value(
        &self,
        attribute: NativeAttribute<'_, 'a>,
    ) -> Result<NativeAttributeValue<'a>, NativeAttributeValueFailure<'a>> {
        let prepare = || {
            Origin::check_original_header(self, &attribute)?;
            let origin = Origin::from_attribute(self, &attribute)?;
            let frame = Frame::original(&origin, &attribute)?;
            let source = prepare_attribute_value(
                self.component().allocator(),
                origin.block.root_source(),
                origin.value_span,
            )
            .map_err(NativeAttributeOperandError::Source)?;
            Ok(NativeAttributeValue {
                origin,
                frame,
                source,
            })
        };
        prepare().map_err(|kind| NativeAttributeValueFailure::original(kind, &attribute))
    }
}

impl<'a> NativeAttributeValue<'a> {
    /// Readonly source coordinates alone do not transfer original-token authority.
    #[must_use]
    pub const fn source(&self) -> EmbedSource<'a> {
        self.source
    }
    #[must_use]
    pub const fn raw_value(&self) -> &'a str {
        self.origin.raw_value
    }
    #[must_use]
    pub const fn name_span(&self) -> Span {
        self.origin.name_span
    }
    /// Complete authored content, excluding quote bytes.
    #[must_use]
    pub const fn value_span(&self) -> Span {
        self.origin.value_span
    }
    /// Complete original value frame, including quotes when present.
    #[must_use]
    pub const fn full_value_span(&self) -> Span {
        self.frame.value
    }
    #[must_use]
    pub const fn equals_span(&self) -> Span {
        self.frame.equals
    }
    #[must_use]
    pub const fn quote_spans(&self) -> Option<(Span, Span)> {
        self.frame.quotes
    }
    /// Short join with the same original selected Component and Attribute token.
    /// Equal authored bytes, ranges or copied source maps cannot mint this view.
    pub fn admitted_for<'s>(
        &'s self,
        selected: &'s NativeTemplateComponent<'a>,
        attribute: NativeAttribute<'s, 'a>,
    ) -> Option<NativeAttributeValueView<'s, 'a>> {
        self.origin
            .matches(selected, &attribute)
            .then_some(NativeAttributeValueView {
                owner: self,
                selected,
                attribute,
            })
    }
    /// Consuming transfer deliberately discards the original header receipt.
    #[must_use]
    pub fn into_source(self) -> EmbedSource<'a> {
        self.source
    }
}

/// An original value observation at its actual current header event.
/// The borrowed join has no File, Element factory or target eligibility authority.
pub struct NativeAttributeValueView<'s, 'a> {
    owner: &'s NativeAttributeValue<'a>,
    selected: &'s NativeTemplateComponent<'a>,
    attribute: NativeAttribute<'s, 'a>,
}
impl<'s, 'a> NativeAttributeValueView<'s, 'a> {
    #[must_use]
    pub fn selected(&self) -> &'s NativeTemplateComponent<'a> {
        self.selected
    }
    #[must_use]
    pub fn attribute(&self) -> &NativeAttribute<'s, 'a> {
        &self.attribute
    }
    #[must_use]
    pub fn observation(&self) -> &'s NativeAttributeValue<'a> {
        self.owner
    }
}

#[cfg(test)]
mod tests;
