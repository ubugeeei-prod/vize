//! Typed refusal evidence, alongside the complete retained original observation.

use vize_l0::{SourceFrameError, Span};
use vize_l1::container::vue::ScriptRole;
use vize_l1::markup::ComponentSourceError;

use super::super::ObservedNativeTemplateRefusal;

/// Genuine descriptor roles outside this first scriptless family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeSfcBlockRole {
    Script(ScriptRole),
    Style,
}

/// Descriptor policy issues/errors stay on the original descriptor owner.
/// Block/frame spans are authored-file coordinates. Template refusals retain
/// their original template-local/decoded/authored coordinate distinctions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeSfcRefusal {
    Descriptor,
    MissingTemplate,
    UnsupportedBlock {
        index: usize,
        span: Span,
        role: NativeSfcBlockRole,
    },
    Component(ComponentSourceError),
    SourceFrame {
        span: Span,
        error: SourceFrameError,
    },
    Template(ObservedNativeTemplateRefusal),
}
