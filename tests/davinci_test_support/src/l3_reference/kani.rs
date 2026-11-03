//! Kani proves the laws of the independent L3 reference on its shipped Rust
//! source: the scheduling contract with the trace semantics, and the C-23
//! graph checker.
//!
//! These harnesses replace the Lean modules `Impeto.ScheduleLaws` and
//! `Impeto.CheckerLaws`; each one names the Lean theorems it replaces. Graphs
//! are unbounded there, so each harness states its bound. The specifications
//! are written from the Lean statements, not from the Rust source.
//!
//! The kernels avoid what CBMC encodes badly: vectors grown, cloned or
//! collected at a symbolic length, references returned from table scans, a
//! machine struct or `&mut` vector threaded through a loop, and slice
//! `contains`. Each costs an order of magnitude in time or memory.

extern crate core;

mod checker;
mod program;
mod schedule;
mod trace;

#[cfg(kani)]
mod checker_proofs;
#[cfg(kani)]
mod graphs;
#[cfg(kani)]
mod schedule_proofs;
