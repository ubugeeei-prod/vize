//! S2 - the Davinci semantic IR (codename Disegno): the pivot stage and the
//! primary consumer surface.
//!
//! **Experimental:** the stage API may change in any alpha release; record
//! intentional breaking changes in the release notes.
//!
//! Named after Leonardo's *disegno*: the drawing that carries the idea,
//! independent of the material it is later executed in.
//!
//! S2 is the normalized, input-neutral representation of UI semantics
//! (`davinci-road/architecture.md`, "S2 — Disegno"). Vue templates, JSX and
//! pug all lower into the same [`op`] family; whatever is genuinely
//! Vue-specific stays a `vue.*` dialect op instead of shaping the core.
//! The Vue template lowering into this family is `vize_s1_to_s2` (codename
//! Ricalco; P2-8); this crate holds the type family, its folio
//! page, the invariants that are enforceable by construction, and the verifier
//! for the ones that are not:
//!
//! - [`op`] — the op enums ([`op::Op`], [`op::BindingOp`]) and their payload
//!   types, arena-resident and **`Drop`-free by construction** through
//!   `vize_s0::{Box, Vec}` (whose const assertion rejects `Drop`
//!   payloads; P1-10 measured it catching two real violations).
//! - [`expr`] — [`expr::ExprRef`], the expression reference: retained JS
//!   AST, foreign dialect (type-only until phase 6), or the classified
//!   escape with pessimal documented semantics (P2-5b's decision; record
//!   in `davinci-road/plan/phase-2-records/p2-5b.md`).
//! - [`folio`] — [`folio::S2Folio`], the S2 stage dump
//!   (`davinci-road/plan/folio-format.md`, "Disegno page").
//! - [`verify`] — the between-pass invariant checks (P2-6), debug/CI only:
//!   local in the GHC `-dcore-lint` sense, and never in a release build.
//! - [`scope`] / [`provenance`] — the side-table vocabulary lowerings
//!   attach beside the tree (P2-8): hygiene scope tags on
//!   binding-introducing ops, and before/after provenance records that
//!   survive failure. Neutral types here so every input dialect's lowering
//!   (Vue today, JSX at P2-16) speaks the same facts.
//!
//! The crate is `no_std + alloc` from birth so S2 artifacts print and parse
//! on any target (wasm32-wasip2 included; P2-14 makes that lane required).

#![no_std]

extern crate alloc;

pub mod expr;
pub mod folio;
pub mod op;
pub mod provenance;
pub mod scope;
pub mod verify;
