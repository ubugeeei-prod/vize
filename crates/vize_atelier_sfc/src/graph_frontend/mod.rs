//! Graph-native products lowered from a cached Relief syntax snapshot.
//!
//! The adapter is deliberately demand-driven. Constructing
//! [`SfcTemplateGraphAdapter`] stores only a borrowed [`ReliefSnapshot`]; Rendu
//! and Flow are built independently only when their corresponding methods are
//! called. This lets Atlas cache Relief once without forcing render, semantic,
//! or control-flow work for syntax-only consumers.

mod adapter;
mod error;
mod expression;
mod flow;
mod flow_control;
mod flow_facts;
mod provenance;
mod rendu;
mod rendu_helpers;

#[cfg(test)]
mod tests;

pub use adapter::SfcTemplateGraphAdapter;
pub use error::SfcGraphAdapterError;
pub use flow::project_relief_snapshot_to_flow;
pub(crate) use flow::project_relief_snapshot_to_flow_with_anchor;
pub use rendu::lower_relief_snapshot_to_rendu;
pub(crate) use rendu::lower_relief_snapshot_to_rendu_with_anchor;
