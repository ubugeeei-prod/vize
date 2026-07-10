//! Provide/Inject analysis.
//!
//! Matches provide() calls with inject() consumers across the component tree.

mod analysis;
mod index;
mod keys;
mod markdown;
mod summary;
mod tree;
mod types;

pub(crate) use analysis::analyze_provide_inject_with_index;
pub(crate) use index::ProvideInjectIndex;
pub use summary::ProvideInjectTreeSummary;
pub(crate) use tree::build_provide_inject_tree_with_index;
pub use types::{ProvideInjectMatch, ProvideInjectTree};

#[cfg(test)]
mod tests;
