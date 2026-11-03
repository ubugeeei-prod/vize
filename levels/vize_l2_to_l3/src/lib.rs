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

pub use crate::partition::dump::{Fact as DumpPartitionFact, Page as PartitionPage};
pub use crate::partition::{PartitionFact, PartitionFacts, PartitionKind};
pub use decision::{
    DecisionBuildError, NativeAnalysis, NativeFileAnalysis, build_decisions, build_dom_decisions,
    build_dom_file_decisions,
};
pub use lower::{Lowered, lower};
pub use vize_l3::jsx::{NativeJsxAnalysis, RejectedJsxAnalysis, build_jsx_decisions};
