//! L2→L3 — the Davinci Impeto lowering.
//!
//! This crate keeps the conversion-library shape: L2 stays independent of L3,
//! L3 stays independent of L2, and this edge owns the lowering plus the shared
//! partition facts later backends consume.
//!
//! **Experimental:** this Phase 3 bridge records allocation and ordering
//! contracts before downstream backends depend on the L3 shape.

#![no_std]

extern crate alloc;

mod lower;
mod partition;

pub use lower::{Lowered, lower};
pub use partition::{
    FolioPartitionFact, L3PartitionFolio, PartitionFact, PartitionFacts, PartitionKind,
};
