//! The lowering entry: L1 surface tree in, L2 artifact out — **total,
//! no rollback** (the MLIR import).
//!
//! [`lower`] never panics and never abandons partial work: every L1
//! construct either becomes an op of the existing L2 family or leaves a
//! [`Diagnostic`] beside the kept fragments, and the three fact channels
//! (diagnostics, provenance, hygiene scopes) are all populated by the
//! time the function returns — including on inputs that are wrong from
//! the first byte. The tokenizer's [`SurfaceError`]s enter the unified
//! channel here (`Stage::Surface`, the exact `ErrorCode` message),
//! ahead of the lowering's own `Stage::Semantic` findings.
//!
//! # Id accounting (the numbering law)
//!
//! Ops are numbered densely in page order as they are decided
//! ([`lower::cx`]); [`Lowered::op_count`] equals
//! `L2Page::of(&lowered.artifact.root.ops).op_count()` on every input —
//! the law the side tables and the L2 verifier's `verify_table` key on.
//!
//! Physical Vue-policy ownership lives under `dialect/vue`; these ordinary
//! child declarations preserve every existing module and type identity.
//!
//! [`Diagnostic`]: vize_l0::diag::Diagnostic
//! [`L2Page`]: vize_l2::dump::Page
//! [`SurfaceError`]: vize_l1::markup::SurfaceError

#[cfg(feature = "davinci-benchmark-profile")]
mod benchmark;
mod binding;
mod bindop;
mod caps;
mod cloak;
mod css;
mod cx;
mod directive;
mod element;
mod expr;
mod finish;
mod foreign;
mod forop;
mod frozen;
mod html;
mod if_keys;
mod leaf;
mod once_memo;
pub mod pug;
mod show;
mod slot;
mod structural;
mod sugar;
mod table;
mod text;
mod vfor;
mod vtext;

include!("dialect/vue/lower.rs");
