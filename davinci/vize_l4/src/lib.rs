//! L4 — emission (codename: Stesura).
//!
//! **Experimental (#6840).** The module layout follows the
//! [#6840 design](https://github.com/ubugeeei-prod/vize/issues/6840#issuecomment-5847908963).
//! The writer, prepared-fragment module assembler and checked supported-expression
//! consumer are implemented. Full binding/grammar and targets
//! remain unfinished; no product selects a complete native route.
//!
//! L4 writes output text directly. There is no JS AST plus codegen step:
//!
//! - [`write`](mod@write) — the one append-only [`write::Writer`] every target uses:
//!   indentation, the used-helper set, and span links that cost nothing when
//!   the writer does not record. Preambles are assembled last, so nothing is
//!   inserted into the middle of the text.
//! - [`expr`] — expression rewriting by span from the L2 identifier-resolution
//!   table. Expressions are parsed once, in L1; L4 never reparses them.
//! - [`runtime`] — helper vocabulary for pinned DOM, SSR and Vapor releases.
//! - [`module`] — SFC module assembly (imports, hoists, script, render,
//!   exports) into one writer, so one source map covers the whole module.
//! - [`targets`] — the DOM, SSR and Vapor emitters and the type-check
//!   projection.
//!
//! The production DOM emitter ([`targets::dom::artifact`]) is the exception to
//! the rules above until the native DOM target replaces it: it reads the
//! canonical L2 artifact and pass facts, re-parses retained expressions it
//! cannot reuse, and erases TypeScript through oxc. Its source-level entry
//! points parse and lower through `vize_l1` and `vize_l1_to_l2`'s public API.
//!
//! Dependencies are level crates (L0, L1, L1→L2, L2, L3), oxc for that DOM
//! emitter, and `serde_json` for Source Map v3 serialization. The production
//! compiler keeps its own emission document until its fix-history gate closes
//! (#6880).

#![cfg_attr(not(test), no_std)]

extern crate alloc;
// The DOM emitter's `typescript` lane reaches oxc's transformer, whose entry
// point takes a `&std::path::Path`; with the feature off nothing links `std`.
#[cfg(all(feature = "typescript", not(test)))]
extern crate std;

pub mod expr;
pub mod framework;
pub mod jsx;
pub mod module;
pub mod registry;
pub mod runtime;
pub mod targets;
pub mod write;
