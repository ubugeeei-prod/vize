//! Normal File storage for the original whole input and actual node outcome.

use super::{FileArtifact, RejectedFile};
use crate::lang::js::{InterpolationInput, TemplateRouteIssueKind};
use vize_l0::id::NodeId;

mod failure;
pub use failure::FileInterpolationFailure;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileInterpolationState {
    Pending,
    Admitted(NodeId),
    Refused(TemplateRouteIssueKind),
}

/// Construction belongs only to the original private File receiver.
/// Pending/refused rows retain the complete normal stock owner, never a fake id.
pub struct FileInterpolation<'a> {
    pub(crate) input: InterpolationInput<'a>,
    pub(crate) state: FileInterpolationState,
}

impl<'a> FileInterpolation<'a> {
    #[must_use]
    pub const fn input(&self) -> &InterpolationInput<'a> {
        &self.input
    }
    #[must_use]
    pub const fn state(&self) -> FileInterpolationState {
        self.state
    }
}

impl<'a> FileArtifact<'a> {
    /// Whole original inputs remain owned across normal admission or refusal.
    #[must_use]
    pub fn file_interpolations(&self) -> &[FileInterpolation<'a>] {
        &self.facts.file_interpolations
    }

    /// Only the node actually minted with this File's original input joins.
    #[must_use]
    pub fn file_interpolation(&self, node: NodeId) -> Option<&FileInterpolation<'a>> {
        let index = *self.facts.native_interpolation_nodes.get(node)?;
        let record = self.facts.file_interpolations.get(index)?;
        matches!(record.state(), FileInterpolationState::Admitted(actual) if actual == node)
            .then_some(record)
    }
}

impl<'a> RejectedFile<'a> {
    #[must_use]
    pub fn file_interpolations(&self) -> &[FileInterpolation<'a>] {
        &self.facts.file_interpolations
    }
}
