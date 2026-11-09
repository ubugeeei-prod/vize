//! Lowering-published text facts.
//!
//! Whitespace condensing and text/interpolation merging already live in
//! [`crate::lower::text`], because both decisions need L1 comment
//! visibility. P2-12b removes the remaining transform walk too: a mixed
//! text run now validates and records its fact when the lowering mints
//! the compound op, then this module only mirrors that side table into
//! [`L2Facts`](super::L2Facts).
//!
//! The old `text` transform pass is deliberately not in the pipeline any
//! more. The invariant checks moved to the construction and legacy-filter
//! mutation sites, where the op span and rebuilt compound spelling are
//! still in hand.

use vize_l0::side_table::SideTable;

use crate::lower::Lowered;

pub use vize_l2::analysis::text_facts::TextFacts;

/// Publish the lowering's already-validated compound text facts without
/// walking the op tree.
#[must_use]
pub(super) fn facts_from_lowering<P>(lowered: &Lowered<'_, P>) -> SideTable<TextFacts> {
    vize_l2::analysis::text_facts::publish_text_facts(&lowered.artifact.texts)
}
