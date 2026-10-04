//! Private complete/refused outcomes retain genuine original owners.

use vize_l1::container::vue::DescriptorObservation;
use vize_l1::markup::{
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
            .field("refusal", &self.refusal())
            .field(
                "has_interpolation_failure",
                &self.interpolation_failure.is_some(),
            )
            .finish()
    }
}

impl<'a> NativeSfcObservation<'a> {
    pub fn source(&self) -> &'a str {
        self.descriptor.source()
    }
    pub fn options(&self) -> NativeSfcOptions {
        self.options
    }
    pub fn descriptor(&self) -> &DescriptorObservation<'a> {
        &self.descriptor
    }
    pub fn selected(&self) -> Option<&NativeTemplateComponent<'a>> {
        self.selected.as_ref()
    }
    pub fn operands(&self) -> &[NativeInterpolationOperand<'a>] {
        &self.operands
    }
    pub fn interpolation_failure(&self) -> Option<&NativeInterpolationFailure<'a>> {
        self.interpolation_failure.as_ref()
    }
    pub fn refusal(&self) -> Option<NativeSfcRefusal> {
        match &self.outcome {
            Outcome::Document(_) => None,
            Outcome::Refused(refusal) => Some(*refusal),
        }
    }
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
