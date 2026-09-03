//! Davinci - the dump/round-trip substrate for the Vize compiler
//! rearchitecture.
//!
//! **Experimental:** the public API and dump format may change in any alpha
//! release; record intentional breaking changes in the release notes.
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
//! The stage IRs themselves land in their own crates (`vize_s2` for S2);
//! see `davinci-road/architecture.md`. New implementation code should prefer
//! the stage aliases recorded in [`stage`] (`vize_s0`, `vize_s1`, `vize_s2`,
//! `vize_s1_to_s2`) over any remaining historical art-name package ids.
//!
//! The crate is `no_std + alloc` from birth so every future stage artifact
//! can print and parse on any target (wasm32-wasip2 included). Host-only
//! helpers belong in binaries (`davinci-opt`), never in the library.

#![no_std]

extern crate alloc;

// `#[derive(Folio)]` expands to `::vize_davinci::...` paths so the same
// expansion works in every consumer; this alias makes those paths resolve
// inside the crate itself.
extern crate self as vize_davinci;

pub mod diagnostic;
pub mod folio;
pub mod id;
pub mod legacy_plan;
pub mod pass;
pub mod side_table;
pub mod stage;
