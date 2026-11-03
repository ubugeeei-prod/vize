//! L1 — the lossless Vue-template surface tree (codename Sinopia, Davinci P2-7).
//!
//! **Experimental:** the stage API may change in any alpha release; record
//! intentional breaking changes in the release notes.
//!
//! A *sinopia* is the preparatory underdrawing beneath a fresco. When the
//! finished layer is detached from the wall, the sinopia survives as the
//! byte-faithful record of what was actually drawn — which is this crate's
//! contract for Vue templates: [`parse`](markup::parse) builds a typed tree
//! whose in-order render is the **exact source bytes**, malformed input
//! included, and a debug verifier asserts that on every construction
//! (`render(tree) == source`, the SwiftSyntax import recorded in
//! `docs/davinci/prior-art-toolchains.md`).
//!
//! The tree is emitted by the shared native profile lexer. The generic parse
//! entry reuses the Vue namespace owner with a neutral syntax policy: its
//! existing no-`v-pre` behavior remains distinct from native dialect admission.
//! This entry now uses the owner's arena-backed ancestry frames; the lexer
//! remains one pass without an added pipeline stage or serialization.
//! Its event stream drives construction without a private re-scan. Armature's
//! published callback facade delegates to the same lexer state machine.
//!
//! # The hole policy (single, documented — L1→L2 consumers read this)
//!
//! Error tolerance is **structural**: every consumer sees one
//! uniformly-shaped tree with typed holes, never a per-consumer error
//! special case. Three clauses cover every malformed byte:
//!
//! 1. **Absent-but-required syntax is a typed `Missing` hole** at the exact
//!    offset where the syntax belongs: an open tag's `>` at EOF
//!    ([`OpenTag::gt`](markup::surface::OpenTag)), a close tag's `>` at EOF
//!    ([`CloseTag::gt`](markup::surface::CloseTag)), an unterminated quoted value's
//!    closing quote ([`AttrValue::close_quote`](markup::surface::AttrValue)), an
//!    announced-but-absent attribute value (`a= >`,
//!    [`AttrValue::content`](markup::surface::AttrValue)) — all
//!    [`TokenStatus::Missing`](markup::surface::TokenStatus) tokens, zero-width —
//!    and a required-but-absent end tag, which is the node-level hole
//!    [`ElementClose::Missing`](markup::surface::ElementClose).
//! 2. **Bytes with no structural home at tree level are a typed
//!    `Unexpected` node** ([`SurfaceChild::Unexpected`](markup::surface::SurfaceChild))
//!    carrying the verbatim slice: a stray end tag that matches no open
//!    element, an empty `</>`, trailing bytes the tokenizer consumed
//!    without reporting structure.
//! 3. **Bytes with no structural home inside a tag ride in the next
//!    token's [`leading`](markup::surface::Token::leading) slice**, under the
//!    diagnostic the tokenizer already reported (a stray `/` in
//!    `<div / a>`, junk between an end-tag name and its `>`). In
//!    well-formed source `leading` is pure inter-token whitespace; it is
//!    verbatim source either way, so fidelity never depends on
//!    well-formedness. This is the one place L1 diverges from
//!    SwiftSyntax's `UnexpectedNodes`-between-any-two-children — recorded
//!    in the P2-7 record with the reasoning.
//!
//! # Strings and residency
//!
//! Every string in the tree is a `&'a str` slice of the input (P1-10: no
//! owned strings in nodes, ever). Containers are `vize_l0::{Box, Vec}`,
//! whose const assertion rejects `Drop` payloads, so the tree is Drop-free
//! and arena-resident by construction. Node footprints are pinned by
//! `#[cfg(target_pointer_width = "64")]`-guarded const asserts in
//! [`markup::surface`].

#![no_std]

extern crate alloc;

pub mod container;
pub mod css;
pub mod dialect;
pub mod edit;
pub mod embed;
pub mod markup;
pub mod pug;

mod build;
mod event;
mod moved;
mod slice;

// The surface was moved under `markup` (#6835); these paths are deprecated.
pub use moved::*;
