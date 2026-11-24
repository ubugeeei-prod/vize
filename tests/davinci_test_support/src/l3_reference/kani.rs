//! Kani proves the laws of the L3 reference incremental update machine on its
//! shipped Rust source. The maintainer kept Kani for this machine (the
//! 2026-11-17 Creusot decision record): its trees recurse through
//! `Vec`, which Creusot accepts only with a trusted declaration. The
//! scheduling contract and the C-23 checker are proved by Creusot instead
//! (`creusot.rs`), for programs of any size.
//!
//! These harnesses replace the Lean module `Impeto.IncrementalLaws`; each one
//! names the Lean theorems it replaces. Their inputs are unbounded there, so
//! each harness states its bound. The specifications are written from the
//! Lean statements, not from the Rust source.
//!
//! The kernel avoids what CBMC encodes badly: vectors grown, cloned or
//! collected at a symbolic length, references returned from table scans, a
//! machine struct or `&mut` vector threaded through a loop, and slice
//! `contains`. Each costs an order of magnitude in time or memory.

extern crate core;

mod incremental;

#[cfg(kani)]
mod incremental_proofs;
