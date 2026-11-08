//! Private complete/refused outcomes retain genuine original owners.

use vize_l1::container::vue::DescriptorObservation;
use vize_l1::markup::{
    NativeAttributeBindingExpression, NativeAttributeExpression, NativeAttributeExpressionFailure,
    NativeInterpolationFailure, NativeInterpolationOperand, NativeTemplateComponent,
};

use super::super::{Doc, print};
use super::{NativeSfcOptions, NativeSfcRefusal};
use crate::FormatResult;

pub(super) enum Outcome<'a> {
    Document(Doc<'a>),
    Refused(NativeSfcRefusal),
}

/// The original descriptor, selected component and observations are owned here.
/// Only the genuine observing constructor can establish a complete full-SFC Doc.
/// Refusals retain every original observation already made and expose no partial Doc.
///
/// ```compile_fail
/// use vize_glyph::native_doc::{Doc, NativeSfcObservation};
/// fn outlive_owner<'a>(owner: NativeSfcObservation<'a>) -> &'a Doc<'a> {
///     owner.document().unwrap()
/// }
/// ```
pub struct NativeSfcObservation<'a> {
    pub(super) descriptor: DescriptorObservation<'a>,
    pub(super) selected: Option<NativeTemplateComponent<'a>>,
    pub(super) operands: std::vec::Vec<NativeInterpolationOperand<'a>>,
    pub(super) interpolation_failure: Option<NativeInterpolationFailure<'a>>,
    pub(super) attributes: std::vec::Vec<NativeAttributeExpression<'a>>,
    pub(super) attribute_failure: Option<NativeAttributeExpressionFailure<'a>>,
    pub(super) bindings: std::vec::Vec<NativeAttributeBindingExpression<'a>>,
    pub(super) binding_failure: Option<NativeAttributeExpressionFailure<'a>>,
    pub(super) options: NativeSfcOptions,
    pub(super) outcome: Outcome<'a>,
}

impl core::fmt::Debug for NativeSfcObservation<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("NativeSfcObservation")
            .field("source_length", &self.source().len())
            .field("options", &self.options)
            .field("has_selected", &self.selected.is_some())
            .field("operand_count", &self.operands.len())
            .field("attribute_count", &self.attributes.len())
            .field("has_attribute_failure", &self.attribute_failure.is_some())
            .field("binding_count", &self.bindings.len())
            .field("has_binding_failure", &self.binding_failure.is_some())
            .field("refusal", &self.refusal())
            .field(
                "has_interpolation_failure",
                &self.interpolation_failure.is_some(),
            )
            .finish()
    }
}

impl<'a> NativeSfcObservation<'a> {
    /// Return the complete authored SFC source retained by the descriptor.
    pub fn source(&self) -> &'a str {
        self.descriptor.source()
    }
    /// Return the native descriptor, print and directive policies used.
    pub fn options(&self) -> NativeSfcOptions {
        self.options
    }
    /// Borrow the original descriptor observation and all of its evidence.
    pub fn descriptor(&self) -> &DescriptorObservation<'a> {
        &self.descriptor
    }
    /// Borrow the original selected template, if selection succeeded.
    pub fn selected(&self) -> Option<&NativeTemplateComponent<'a>> {
        self.selected.as_ref()
    }
    /// Borrow interpolation operands observed before completion or refusal.
    pub fn operands(&self) -> &[NativeInterpolationOperand<'a>] {
        &self.operands
    }
    /// Borrow conditional directive operands observed before completion or refusal.
    pub fn attribute_operands(&self) -> &[NativeAttributeExpression<'a>] {
        &self.attributes
    }
    /// Borrow the conditional directive observation failure, if one occurred.
    pub fn attribute_failure(&self) -> Option<&NativeAttributeExpressionFailure<'a>> {
        self.attribute_failure.as_ref()
    }
    /// Borrow binding directive operands observed before completion or refusal.
    pub fn binding_operands(&self) -> &[NativeAttributeBindingExpression<'a>] {
        &self.bindings
    }
    /// Borrow the binding directive observation failure, if one occurred.
    pub fn binding_failure(&self) -> Option<&NativeAttributeExpressionFailure<'a>> {
        self.binding_failure.as_ref()
    }
    /// Borrow the interpolation observation failure, if one occurred.
    pub fn interpolation_failure(&self) -> Option<&NativeInterpolationFailure<'a>> {
        self.interpolation_failure.as_ref()
    }
    /// Return the typed refusal retained for an incomplete observation.
    pub fn refusal(&self) -> Option<NativeSfcRefusal> {
        match &self.outcome {
            Outcome::Document(_) => None,
            Outcome::Refused(refusal) => Some(*refusal),
        }
    }
    /// Borrow the complete document or return the retained refusal.
    pub fn document(&self) -> Result<&Doc<'a>, NativeSfcRefusal> {
        match &self.outcome {
            Outcome::Document(document) => Ok(document),
            Outcome::Refused(refusal) => Err(*refusal),
        }
    }
    /// Print the complete original-root Doc using captured native options.
    /// This performs no parsing, observation, postprint trimming or fallback.
    pub fn format(&self) -> Result<FormatResult, NativeSfcRefusal> {
        let code = print(self.document()?, &self.options.print);
        Ok(FormatResult {
            changed: code != self.source(),
            code,
        })
    }
}
