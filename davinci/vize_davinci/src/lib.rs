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
//!   every renderer reads, with the witness law and precision tiers as types.
//! - [`witness`] — TS-36: re-checking a diagnostic's witness chain against the
//!   fact base.
//! - [`fact`] — the fact API: fact groups, static demand declarations, the
//!   stratified registry and the [`FactManager`](fact::FactManager).
//! - [`pass`] — the pass manager: pipelines as const data, classified and
//!   fused at build time.
//! - [`legacy_plan`] — the shipped backends' template traversals, declared as
//!   plans so a migration has something to be measured against.
//! - [`dump`] — the textual stage-dump contract (`trait Dump`).
//! - [`key`] — [`ArtifactKey`](key::ArtifactKey), the span-relative content
//!   identity every cache of a stage artifact keys on (P5-1a).
//! - [`summary`] — [`SfcSummary`](summary::SfcSummary), the per-SFC interface
//!   fingerprinted per declaration (P5-2).
//! - [`render`] — the rustc/Elm-grade terminal renderer every diagnostic
//!   surface shares, localized through a caller-supplied catalog.
//!
//! The stage IRs themselves land in their own crates (`vize_l2` for L2);
//! see `docs/davinci/architecture.md`. New implementation code should prefer
//! the stage aliases recorded in [`stage`] (`vize_l0`, `vize_l1`, `vize_l2`,
//! `vize_l1_to_l2`) over any remaining historical art-name package ids.
//!
//! The crate is `no_std + alloc` from birth so every future stage artifact
//! can print and parse on any target (wasm32-wasip2 included). Host-only
//! helpers belong in the `vize` host CLI, never in this library.

#![no_std]

extern crate alloc;

// `#[derive(Dump)]` expands to `::vize_davinci::...` paths so the same
// expansion works in every consumer; this alias makes those paths resolve
// inside the crate itself.
extern crate self as vize_davinci;

pub use vize_l0::diag as diagnostic;
pub mod dump;
pub use vize_l0::fact;
pub use vize_l0::id;
pub use vize_l0::key;
pub mod legacy_plan;
pub use vize_l0::pass;
pub mod render;
pub use vize_l0::side_table;
pub mod stage;
pub mod summary;
pub use vize_l0::diag::verify as witness;

pub use vize_l0::assert_dump_snapshot;
