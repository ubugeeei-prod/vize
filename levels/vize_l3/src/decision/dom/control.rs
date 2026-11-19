//! Checked source-order conditional facts; target syntax stays in L4.

use alloc::vec::Vec;
use vize_l0::id::NodeId;
use vize_l2::op::{IfBranch, IfOp};

use super::ValueKind;

pub(super) mod build;

/// A branch's checked source-order identity within this exact conditional.
///
/// Callers cannot mint a target key independently:
/// ```compile_fail
/// use vize_l3::decision::dom::control::BranchDiscriminant;
/// let _ = BranchDiscriminant(0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BranchDiscriminant(pub(super) u32);

impl BranchDiscriminant {
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

/// The admitted branch root's semantic block ownership.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomBlockEligibility {
    NativeElement,
}

/// Whether the conditional has an authored final branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionalFallback {
    Placeholder,
    AuthoredElse,
}

/// One retained branch header and its actual single canonical root id.
#[derive(Debug)]
pub struct DomConditionalBranch<'owner, 'arena> {
    pub(super) owner: &'owner IfBranch<'arena>,
    pub(super) discriminant: BranchDiscriminant,
    pub(super) root: NodeId,
    pub(super) condition: Option<ValueKind>,
    pub(super) block: DomBlockEligibility,
}

impl<'owner, 'arena> DomConditionalBranch<'owner, 'arena> {
    #[must_use]
    pub fn owner(&self) -> &'owner IfBranch<'arena> {
        self.owner
    }

    #[must_use]
    pub const fn discriminant(&self) -> BranchDiscriminant {
        self.discriminant
    }

    #[must_use]
    pub const fn root(&self) -> NodeId {
        self.root
    }

    #[must_use]
    pub const fn condition(&self) -> Option<ValueKind> {
        self.condition
    }

    #[must_use]
    pub const fn block(&self) -> DomBlockEligibility {
        self.block
    }
}

/// Sparse facts sealed by the sole owner-bound native analysis producer.
///
/// Private fields prevent callers from manufacturing branch identities or
/// associating foreign root ids with a borrowed conditional payload.
#[derive(Debug)]
pub struct DomConditional<'owner, 'arena> {
    pub(super) owner: &'owner IfOp<'arena>,
    pub(super) branches: Vec<DomConditionalBranch<'owner, 'arena>>,
    pub(super) fallback: ConditionalFallback,
}

impl<'owner, 'arena> DomConditional<'owner, 'arena> {
    #[must_use]
    pub fn owner(&self) -> &'owner IfOp<'arena> {
        self.owner
    }

    #[must_use]
    pub fn branches(&self) -> &[DomConditionalBranch<'owner, 'arena>] {
        &self.branches
    }

    #[must_use]
    pub const fn fallback(&self) -> ConditionalFallback {
        self.fallback
    }
}
