//! Kani proves the laws of the independent L3 reference on its shipped Rust
//! source: the scheduling contract with the trace semantics, the C-23 graph
//! checker and the TS-29 update machine.
//!
//! These harnesses replace the Lean modules `Impeto.ScheduleLaws`,
//! `Impeto.CheckerLaws` and `Impeto.IncrementalLaws`; each one names the Lean
//! theorems it replaces. Graphs, lists and trees are unbounded there, so each
//! harness states its bound. The specifications are written from the Lean
//! statements, not from the Rust source.

extern crate core;

mod checker;
mod incremental;
mod program;
mod schedule;
mod trace;

#[cfg(kani)]
mod checker_proofs;
#[cfg(kani)]
mod graphs;
#[cfg(kani)]
mod incremental_proofs;
#[cfg(kani)]
mod schedule_proofs;
