//! Typed refusal evidence, alongside the complete retained original observation.

use vize_l0::{SourceFrameError, Span};
use vize_l1::container::vue::ScriptRole;
use vize_l1::markup::ComponentSourceError;

use super::super::ObservedNativeTemplateRefusal;

/// Genuine descriptor roles outside this first scriptless family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeSfcBlockRole {
    /// A script block, with its original descriptor role.
    Script(ScriptRole),
    /// A style block, which is outside the scriptless native family.
    Style,
}

/// Descriptor policy issues/errors stay on the original descriptor owner.
/// Block/frame spans are authored-file coordinates. Template refusals retain
/// their original template-local/decoded/authored coordinate distinctions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeSfcRefusal {
    /// The descriptor observation itself refused or could not select safely.
    Descriptor,
    /// The descriptor has no template block to format.
    MissingTemplate,
    /// A top-level block is outside the supported scriptless family.
    UnsupportedBlock {
        /// Zero-based descriptor block ordinal.
        index: usize,
        /// Authored-file span of the unsupported block.
        span: Span,
        /// Semantic role of the unsupported block.
        role: NativeSfcBlockRole,
    },
    /// The selected template could not provide a complete source component.
    Component(ComponentSourceError),
    /// A descriptor range could not be framed in the authored source.
    SourceFrame {
        /// Authored-file span whose frame was invalid.
        span: Span,
        /// Original source-frame error.
        error: SourceFrameError,
    },
    /// The selected template or one of its observed expressions refused.
    Template(ObservedNativeTemplateRefusal),
}
