//! Lowering-published `v-if` branch-key facts.
//!
//! P2-9 originally kept a mandatory `v-if` transform pass whose only
//! remaining work was to lift branch-carrier `key` props into
//! [`IfFacts`] and diagnose duplicate keys. The lowering now performs
//! that work where the chain, branch order, wrapper captures, and
//! carrier surface are already available, and the table rides on the
//! canonical artifact (`ArtifactParts::if_facts`); no transform walk or
//! copy remains.

pub use crate::lower::SAME_KEY_MESSAGE;
pub use crate::lower::{BranchKey, BranchKeyKind, IfFacts};

/// The pass name kept for existing folio/report strings.
pub const NAME: &str = "v-if";
