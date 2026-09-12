//! S2→S3 — the Davinci Impeto lowering.
//!
//! This crate keeps the conversion-library shape: S2 stays independent of S3,
//! S3 stays independent of S2, and this edge owns the lowering plus the shared
//! partition facts later backends consume.
//!
//! **Experimental:** this Phase 3 bridge records allocation and ordering
//! contracts before downstream backends depend on the S3 shape.

#![no_std]

extern crate alloc;

mod lower;
mod partition;

pub use lower::{Lowered, lower};
pub use partition::{PartitionFact, PartitionFacts, PartitionKind};
