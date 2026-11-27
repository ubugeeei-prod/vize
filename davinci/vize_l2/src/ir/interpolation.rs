//! Normal File storage for the original whole input and actual node outcome.

use super::{L2Ir, RejectedL2Ir};
use crate::lang::js::{NativeInterpolationInput, NativeTemplateIssueKind};
use vize_l0::id::NodeId;

mod failure;
pub use failure::NativeL2IrInterpolationFailure;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeL2IrInterpolationState {
    Pending,
    Admitted(NodeId),
    Refused(NativeTemplateIssueKind),
}

/// Construction belongs only to the original private File receiver.
/// Pending/refused rows retain the complete normal stock owner, never a fake id.
pub struct NativeL2IrInterpolation<'a> {
    pub(crate) input: NativeInterpolationInput<'a>,
    pub(crate) state: NativeL2IrInterpolationState,
}

impl<'a> NativeL2IrInterpolation<'a> {
    #[must_use]
    pub const fn input(&self) -> &NativeInterpolationInput<'a> {
        &self.input
    }
    #[must_use]
    pub const fn state(&self) -> NativeL2IrInterpolationState {
        self.state
    }
}

impl<'a> L2Ir<'a> {
    /// Whole original inputs remain owned across normal admission or refusal.
    #[must_use]
    pub fn native_interpolations(&self) -> &[NativeL2IrInterpolation<'a>] {
        &self.facts.native_interpolations
    }

    /// Only the node actually minted with this File's original input joins.
    #[must_use]
    pub fn native_interpolation(&self, node: NodeId) -> Option<&NativeL2IrInterpolation<'a>> {
        let index = *self.facts.native_interpolation_nodes.get(node)?;
        let record = self.facts.native_interpolations.get(index)?;
        matches!(record.state(), NativeL2IrInterpolationState::Admitted(actual) if actual == node)
            .then_some(record)
    }
}

impl<'a> RejectedL2Ir<'a> {
    #[must_use]
    pub fn native_interpolations(&self) -> &[NativeL2IrInterpolation<'a>] {
        &self.facts.native_interpolations
    }
}
