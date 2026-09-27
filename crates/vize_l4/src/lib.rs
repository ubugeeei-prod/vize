//! L4 — emission (codename: Stesura).
//!
//! **Experimental skeleton (#6840).** The module layout and signatures follow
//! the [#6840 design](https://github.com/ubugeeei-prod/vize/issues/6840#issuecomment-5847908963);
//! unfinished bodies are explicit `todo!()`s and no product reaches them.
//!
//! L4 writes output text directly. There is no JS AST plus codegen step:
//!
//! - [`write`](mod@write) — the one append-only [`write::Writer`] every target uses:
//!   indentation, the used-helper set, and span links that cost nothing when
//!   the writer does not record. Preambles are assembled last, so nothing is
//!   inserted into the middle of the text.
//! - [`expr`] — expression rewriting by span from the L2 identifier-resolution
//!   table. Expressions are parsed once, in L1; L4 never reparses them.
//! - [`runtime`] — the runtime helper vocabulary per runtime.
//! - [`module`] — SFC module assembly (imports, hoists, script, render,
//!   exports) into one writer, so one source map covers the whole module.
//! - [`targets`] — the DOM, SSR and Vapor emitters and the type-check
//!   projection.
//!
//! Dependencies are level crates only (L0, L2, L3). Legacy codegen depends on
//! this crate, never the reverse.

#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod expr;
pub mod module;
pub mod runtime;
pub mod targets;
pub mod write;
