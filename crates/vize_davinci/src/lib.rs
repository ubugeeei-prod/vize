//! Davinci - the dump/round-trip substrate for the Vize compiler
//! rearchitecture.
//!
//! Named after Leonardo's manuscripts, whose folios carried both the drawing
//! and the notes needed to read it back.
//!
//! The crate hosts the substrate every stage keys on, and nothing
//! stage-specific:
//!
//! - [`id`] — [`NodeId`](id::NodeId), the identity cross-stage references and
//!   side tables use.
//! - [`side_table`] — [`SideTable`](side_table::SideTable), analysis results
//!   stored beside the tree rather than on fat nodes.
//! - [`diagnostic`] — [`Diagnostic`](diagnostic::Diagnostic), the one channel
//!   every renderer reads.
//! - [`pass`] — the pass manager: pipelines as const data, classified and
//!   fused at build time.
//! - [`legacy_plan`] — the shipped backends' template traversals, declared as
//!   plans so a migration has something to be measured against.
//! - [`folio`] — the textual stage-dump contract (`trait Folio`).
//!
//! The stage IRs themselves land in their own crates (`vize_disegno` for S2);
//! see `davinci-road/architecture.md`.
//!
//! The crate is `no_std + alloc` from birth so every future stage artifact
//! can print and parse on any target (wasm32-wasip2 included). Host-only
//! helpers belong in binaries (`davinci-opt`), never in the library.

#![no_std]

extern crate alloc;

pub mod diagnostic;
pub mod folio;
pub mod id;
pub mod legacy_plan;
pub mod pass;
pub mod side_table;
