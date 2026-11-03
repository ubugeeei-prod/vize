//! Static-subtree analysis over canonical L2 owners.
//!
//! The analysis publishes the existing per-owner lattice and ordered
//! provenance without mutating its region. It consumes retained expressions
//! and the shared page-order walk; no dialect legalizer or backend is needed.
//! Hoisting decisions remain position- and option-dependent in the backend.
//! The transitional Vue pass is only an adapter to this implementation.

use vize_l0::pass::{Fusability, NoRemarks, PassDesc, PassKind, Preserved, RemarkSink};
use vize_l0::side_table::SideTable;

use crate::op::{Namespace, Region};
use crate::provenance::ProvenanceSink;
use crate::walk::{PageWalk, assert_accounting};

mod consts;
mod lattice;
mod remarks;

pub use super::constant_expression::is_self_bound_constant as constant_for_hoist;
pub use remarks::{STATIC_PROPS, STATIC_SUBTREE};

/// The pass name in pipeline strings and folio pages.
pub const NAME: &str = "hoist-static";

/// Static-subtree analysis is optional, fusable and preserves the artifact.
pub const DESC: PassDesc = PassDesc::new(
    NAME,
    PassKind::Optional,
    Fusability::Fusable,
    Preserved::ALL,
);

/// The per-node constant level inherited from the legacy static-type lattice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaticLevel {
    /// Dynamic surface or dynamic descendants; never hoistable whole.
    NotStatic,
    /// Static but for interpolation children; never whole-hoistable.
    HasDynamicText,
    /// The whole subtree is static where backend position policy permits it.
    FullyStatic,
}

/// Static-analysis facts for one canonical L2 element or component owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaticFacts {
    /// The owner's static level.
    pub level: StaticLevel,
    /// Whether the non-empty props surface is fully hoistable.
    pub props_hoistable: bool,
    /// Whether the owner has non-empty, entirely static nested children.
    pub nested_static: bool,
    /// Whether every descendant is a native element or text-family node.
    pub native_descendants: bool,
    /// Whether the owner stands in a foreign markup namespace.
    pub foreign: bool,
}

// Facts cross compile boundaries with their artifact.
const _: () = {
    const fn assert_owned<T: 'static>() {}
    assert_owned::<StaticFacts>();
};

/// Five bytes on every width: one tag byte plus four bools.
const _: () = assert!(core::mem::size_of::<StaticFacts>() == 5);

/// Analyze one complete region using its existing page-order node count.
///
/// Facts and provenance have the original post-order publication order. The
/// region and existing provenance prefix are retained unchanged.
///
/// # Panics
///
/// Panics on mismatched page accounting, which is a producer invariant.
#[must_use]
pub fn run(
    root: &Region<'_>,
    op_count: u32,
    provenance: &mut impl ProvenanceSink,
) -> SideTable<StaticFacts> {
    run_remarked(root, op_count, provenance, &mut NoRemarks)
}

/// [`run`] with the existing static-subtree and static-props remarks.
///
/// A detached sink leaves the analysis and its provenance identical.
///
/// # Panics
///
/// As [`run`].
#[must_use]
pub fn run_remarked<R: RemarkSink>(
    root: &Region<'_>,
    op_count: u32,
    provenance: &mut impl ProvenanceSink,
    remarks: &mut R,
) -> SideTable<StaticFacts> {
    let mut facts = SideTable::new();
    let mut walk = PageWalk::new();
    lattice::visit_region(
        &mut walk,
        &root.ops,
        Namespace::Html,
        provenance,
        &mut facts,
        remarks,
    );
    assert_accounting(&walk, op_count, NAME);
    facts
}

#[cfg(test)]
mod tests;
