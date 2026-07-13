//! Graph-native products lowered from a cached Relief syntax snapshot.
//!
//! The adapter is deliberately demand-driven. Constructing
//! [`TemplateGraphAdapter`] stores only a borrowed [`ReliefSnapshot`]; Rendu
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
mod scope;

#[cfg(test)]
mod builtin_tests;
#[cfg(test)]
mod tests;

pub use adapter::TemplateGraphAdapter;
pub use error::TemplateGraphAdapterError;
pub use flow::project_relief_snapshot_to_flow;
#[doc(hidden)]
pub use flow::project_relief_snapshot_to_flow_with_anchor;
pub use rendu::lower_relief_snapshot_to_rendu;
#[doc(hidden)]
pub use rendu::{
    lower_relief_snapshot_to_rendu_with_anchor,
    lower_relief_snapshot_to_rendu_with_anchor_and_bindings,
};
