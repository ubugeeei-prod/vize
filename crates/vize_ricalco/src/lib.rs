//! Ricalco — the S1→S2 Vue lowering (Davinci P2-8).
//!
//! A *ricalco* is the tracing that transfers a drawing onto a new
//! surface. This crate traces the lossless Vue surface tree (S1,
//! [`vize_sinopia`]) into the neutral semantic op family (S2,
//! [`vize_disegno`]): [`lower`] is a **total function** — every input
//! yields S2 ops and/or [`Diagnostic`]s, never a panic and never a
//! partial-then-abandoned state (the MLIR one-shot import,
//! `davinci-road/prior-art.md`).
//!
//! # The home decision (dependency direction)
//!
//! The lowering is its own crate — the MLIR conversion-library shape —
//! because both alternative homes point a stage at the other stage's
//! internals: inside `vize_disegno` the neutral pivot would depend on the
//! Vue S1 surface (and drag `vize_armature`/`vize_relief` into the
//! required TS-24 wasm lane); inside `vize_sinopia` the lossless record
//! crate would grow semantic decisions and an oxc parser edge. Here the
//! conversion depends downward on both and neither learns of the other;
//! `cargo tree -i` on either stage names only this experimental,
//! `publish = false` crate, so no shipped compile path can observe any of
//! it. Full reasoning in `davinci-road/plan/phase-2-records/p2-8.md`.
//!
//! # What lowering means here
//!
//! - **Structure**: elements, components, text, interpolations, `v-if`
//!   sibling chains into region-owning [`IfOp`]s, `v-for` into [`ForOp`]s
//!   (the split is Vue's text grammar, never a JS parse of the whole
//!   value — the P2-5b `ForValue` decision, consumed not re-derived),
//!   `<slot>` outlets into [`SlotOp`]s, `v-model` into [`ModelOp`]
//!   contracts, custom directives into [`VueDirectiveOp`]s.
//! - **Totality over holes**: the S1 hole policy's typed `Missing` /
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
//!   binding-introducing op ([`vize_disegno::scope`]) and fully
//!   materialized before/after provenance ([`vize_disegno::provenance`])
//!   that both survive failure, Lean-InfoTree style.
//!
//! [`Diagnostic`]: vize_davinci::diagnostic::Diagnostic
//! [`SurfaceError`]: vize_sinopia::SurfaceError
//! [`IfOp`]: vize_disegno::op::IfOp
//! [`ForOp`]: vize_disegno::op::ForOp
//! [`SlotOp`]: vize_disegno::op::SlotOp
//! [`ModelOp`]: vize_disegno::op::ModelOp
//! [`VueDirectiveOp`]: vize_disegno::op::VueDirectiveOp

#![no_std]

extern crate alloc;

pub mod lower;
pub mod pass;

#[cfg(feature = "_legacy")]
pub use lower::{LegacyVueLine, lower_legacy};
pub use lower::{Lowered, lower};
