//! The provenance-generic passes over a recording lowering, compiled once.
//!
//! [`run_transform_with_pass_hook`](super::run_transform_with_pass_hook) is
//! generic over its observer and hook, so each consumer crate compiles its
//! own copy of it. Calling the passes through these non-generic functions
//! keeps the pass bodies out of those copies: they are compiled here, once.

use vize_l0::side_table::SideTable;
use vize_l2::analysis::pass_facts::FilterAssets;

use super::{ModelFacts, SlotFacts, legacy, vmodel, vslot};
use crate::lower::Lowered;

pub(super) fn legacy(lowered: &mut Lowered<'_>) -> FilterAssets {
    legacy::run(lowered)
}

pub(super) fn slots(lowered: &mut Lowered<'_>) -> SideTable<SlotFacts> {
    vslot::run(lowered)
}

pub(super) fn models(lowered: &mut Lowered<'_>) -> SideTable<ModelFacts> {
    vmodel::run(lowered)
}
