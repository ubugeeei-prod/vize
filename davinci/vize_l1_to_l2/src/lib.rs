//! L1→L2 — the Vue lowering (codename Ricalco, Davinci P2-8).
//!
//! **Experimental:** the lowering API may change in any alpha release; record
//! intentional breaking changes in the release notes.
//!
//! A *ricalco* is the tracing that transfers a drawing onto a new
//! surface. This crate traces the lossless Vue surface tree (L1,
//! [`vize_l1`]) into the neutral semantic op family (L2,
//! [`vize_l2`]): [`lower`] is a **total function** — every input
//! yields L2 ops and/or [`Diagnostic`]s, never a panic and never a
//! partial-then-abandoned state (the MLIR one-shot import,
//! `docs/davinci/prior-art.md`).
//!
//! # The home decision (dependency direction)
//!
//! The lowering is its own crate — the MLIR conversion-library shape —
//! because both alternative homes point a stage at the other stage's
//! internals: inside `vize_l2` the neutral pivot would depend on the
//! Vue L1 surface (and drag `vize_armature`/`vize_relief` into the
//! required TS-24 wasm lane); inside `vize_l1` the lossless record
//! crate would grow semantic decisions and an oxc parser edge. Here the
//! conversion depends downward on both and neither learns of the other;
//! `cargo tree -i` on either stage names only this experimental,
//! `publish = false` crate, so no shipped compile path can observe any of
//! it. Full reasoning in `docs/davinci/plan/phase-2-records/p2-8.md`.
//!
//! # What lowering means here
//!
//! - **Structure**: elements, components, text, interpolations, `v-if`
//!   sibling chains into region-owning [`IfOp`]s, `v-for` into [`ForOp`]s
//!   (the split is Vue's text grammar, never a JS parse of the whole
//!   value — the P2-5b `ForValue` decision, consumed not re-derived),
//!   `<slot>` outlets into [`SlotOp`]s, `v-model` into [`ModelOp`]
//!   contracts, custom directives into [`VueDirectiveOp`]s.
//! - **Totality over holes**: the L1 hole policy's typed `Missing` /
//!   `Unexpected` nodes lower structurally — an unclosed element still
//!   produces its op, stray bytes are dropped under provenance — and the
//!   tokenizer's [`SurfaceError`]s enter the unified [`Diagnostic`]
//!   channel here (the P2-7 hand-off).
//! - **Unmappable means diagnostic, never a new op**: whatever the
//!   existing op family cannot carry (`v-bind`/`v-on` before P2-9's
//!   normalized binding ops, `v-slot` content grouping, ...) becomes a
//!   [`Diagnostic`] plus a kept fragment. Adding op variants is P2-9's
//!   decision, not this crate's.
//! - **Facts beside the tree**: hygiene scope tags on every
//!   binding-introducing op ([`vize_l2::scope`]) and before/after
//!   provenance ([`vize_l2::provenance`]), kept for the consumers that
//!   read it; both survive failure, Lean-InfoTree style.
//!
//! [`Diagnostic`]: vize_l0::diag::Diagnostic
//! [`SurfaceError`]: vize_l1::markup::SurfaceError
//! [`IfOp`]: vize_l2::op::IfOp
//! [`ForOp`]: vize_l2::op::ForOp
//! [`SlotOp`]: vize_l2::op::SlotOp
//! [`ModelOp`]: vize_l2::op::ModelOp
//! [`VueDirectiveOp`]: vize_l2::op::VueDirectiveOp

#![no_std]

extern crate alloc;
pub mod dialect;
pub mod exemptions;
mod expression_coordinates;
#[cfg(any())]
mod lang;
mod policy;
pub use policy::*;
pub mod key;
pub mod lower;
pub mod native;
pub mod native_file;
pub mod native_vue2;
pub mod partial;
pub mod pass;
pub mod registry;
pub mod vue_file;

pub use lower::{
    ForeignDialect, Lowered, lower, lower_preserving_comments, lower_source_block,
    lower_source_block_with_caps, lower_source_block_with_foreign_expressions, lower_style_block,
    lower_style_block_in, lower_with_caps, lower_with_frozen_element_spans,
};
