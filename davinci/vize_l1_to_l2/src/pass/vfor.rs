//! Lowering-published `v-for` facts.
//!
//! P2-9 originally kept a preserving mandatory `v-for` pass whose only
//! work was to rewalk L2, validate the lowering's scope table, and
//! publish the consumed `(value, key, index)` view. That validation now
//! happens at the lowering site that mints the `ui.for` scope; this
//! module is the compatibility mirror that presents the same fact table
//! to transform consumers without spending another L2 walk.
//!
//! The important law remains the same: every `ui.for` with an id has one
//! fresh scope tag, its named positions byte-equal the recorded
//! [`ScopeFacts`], and destructuring or escape-classified positions are
//! pessimistic [`ForName::Pending`] rather than synthesized names.
//!
//! [`ScopeFacts`]: vize_l2::scope::ScopeFacts

use vize_l0::side_table::SideTable;

use crate::lower::Lowered;

pub use vize_l2::analysis::for_facts::{ForName, ForParts as ForFacts};

/// The pass name kept for existing folio/report strings.
pub const NAME: &str = "v-for";

/// Mirror lowering-published `ui.for` facts for transform consumers.
#[must_use]
pub fn facts_from_lowering(lowered: &Lowered<'_>) -> SideTable<ForFacts> {
    lowered.for_facts.clone()
}
