//! Normal File storage for the original whole input and actual node outcome.

use super::{FileArtifact, RejectedFile};
use crate::lang::js::{NativeInterpolationInput, NativeTemplateIssueKind};
use vize_l0::id::NodeId;

mod failure;
pub use failure::NativeFileInterpolationFailure;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeFileInterpolationState {
    Pending,
    Admitted(NodeId),
    Refused(NativeTemplateIssueKind),
}

/// Construction belongs only to the original private File receiver.
/// Pending/refused rows retain the complete normal stock owner, never a fake id.
pub struct NativeFileInterpolation<'a> {
    pub(crate) input: NativeInterpolationInput<'a>,
    pub(crate) state: NativeFileInterpolationState,
}

impl<'a> NativeFileInterpolation<'a> {
    #[must_use]
    pub const fn input(&self) -> &NativeInterpolationInput<'a> {
        &self.input
    }
    #[must_use]
    pub const fn state(&self) -> NativeFileInterpolationState {
        self.state
    }
}

impl<'a> FileArtifact<'a> {
    /// Whole original inputs remain owned across normal admission or refusal.
    #[must_use]
    pub fn native_interpolations(&self) -> &[NativeFileInterpolation<'a>] {
        &self.facts.native_interpolations
    }

    /// Only the node actually minted with this File's original input joins.
    #[must_use]
    pub fn native_interpolation(&self, node: NodeId) -> Option<&NativeFileInterpolation<'a>> {
        let index = *self.facts.native_interpolation_nodes.get(node)?;
        let record = self.facts.native_interpolations.get(index)?;
        matches!(record.state(), NativeFileInterpolationState::Admitted(actual) if actual == node)
            .then_some(record)
    }
}

impl<'a> RejectedFile<'a> {
    #[must_use]
    pub fn native_interpolations(&self) -> &[NativeFileInterpolation<'a>] {
        &self.facts.native_interpolations
    }
}
