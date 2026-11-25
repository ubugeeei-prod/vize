//! L2→L3 — the Davinci Impeto lowering.
//!
//! This crate keeps the conversion-library shape: L2 stays independent of L3,
//! the edge owns flat lowering and its partition facts. Native shared decisions
//! are owned by L3 and retain a borrow of their canonical L2 artifact.
//!
//! **Experimental:** this Phase 3 bridge records allocation and ordering
//! contracts before downstream backends depend on the L3 shape.

#![no_std]

extern crate alloc;

pub mod decision;
mod lower;
pub mod partition;
pub mod registry;

pub use crate::partition::dump::{Page as PartitionPage, PartitionRecord as DumpPartitionRecord};
pub use crate::partition::{PartitionAnalysis, PartitionInfo, PartitionKind};
pub use decision::{
    DecisionAnalysis, DecisionBuildError, FileAnalysis, build_decisions, build_dom_decisions,
    build_dom_file_decisions,
};
pub use lower::{Lowered, lower};
pub use vize_l3::jsx::{JsxAnalysis, RejectedJsxAnalysis, build_jsx_decisions};

#[cfg(feature = "graph-queries")]
mod graph;
#[cfg(feature = "graph-queries")]
pub use graph::lower_ir;
