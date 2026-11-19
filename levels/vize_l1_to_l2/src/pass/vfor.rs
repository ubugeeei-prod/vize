//! Lowering-published `v-for` facts.
//!
//! P2-9 originally kept a preserving mandatory `v-for` pass whose only
//! work was to rewalk L2, validate the lowering's scope table, and
//! publish the consumed `(value, key, index)` view. That validation now
//! happens at the lowering site that mints the `ui.for` scope, and the
//! table rides on the canonical artifact (`ArtifactParts::for_facts`); no
//! transform walk or copy remains.
//!
//! The important law remains the same: every `ui.for` with an id has one
//! fresh scope tag, its named positions byte-equal the recorded
//! [`ScopeFacts`], and destructuring or escape-classified positions are
//! pessimistic [`ForName::Pending`] rather than synthesized names.
//!
//! [`ScopeFacts`]: vize_l2::scope::ScopeFacts

pub use vize_l2::analysis::for_facts::{ForName, ForParts as ForFacts};

/// The pass name kept for existing folio/report strings.
pub const NAME: &str = "v-for";
