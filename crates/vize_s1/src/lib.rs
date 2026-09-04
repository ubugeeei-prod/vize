//! S1 — the lossless Vue-template surface tree (codename Sinopia, Davinci P2-7).
//!
//! **Experimental:** the stage API may change in any alpha release; record
//! intentional breaking changes in the release notes.
//!
//! A *sinopia* is the preparatory underdrawing beneath a fresco. When the
//! finished layer is detached from the wall, the sinopia survives as the
//! byte-faithful record of what was actually drawn — which is this crate's
//! contract for Vue templates: [`parse`](parse::parse) builds a typed tree
//! whose in-order render is the **exact source bytes**, malformed input
//! included, and a debug verifier asserts that on every construction
//! (`render(tree) == source`, the SwiftSyntax import recorded in
//! `davinci-road/prior-art-toolchains.md`).
//!
//! The tree is *emitted by `vize_armature`* in the P2-7 contract's sense:
//! armature's tokenizer drives construction (one shared lexer, no private
//! re-scan), while the tree types live here so the experimental S1 surface
//! never enters a published crate's release graph. See the P2-7 record for
//! why the alternative shapes (a module inside `vize_armature`, a layer over
//! the `vize_relief` AST) were not taken.
//!
//! # The hole policy (single, documented — S1→S2 consumers read this)
//!
//! Error tolerance is **structural**: every consumer sees one
//! uniformly-shaped tree with typed holes, never a per-consumer error
//! special case. Three clauses cover every malformed byte:
//!
//! 1. **Absent-but-required syntax is a typed `Missing` hole** at the exact
//!    offset where the syntax belongs: an open tag's `>` at EOF
//!    ([`OpenTag::gt`](surface::OpenTag)), a close tag's `>` at EOF
//!    ([`CloseTag::gt`](surface::CloseTag)), an unterminated quoted value's
//!    closing quote ([`AttrValue::close_quote`](surface::AttrValue)), an
//!    announced-but-absent attribute value (`a= >`,
//!    [`AttrValue::content`](surface::AttrValue)) — all
//!    [`TokenStatus::Missing`](surface::TokenStatus) tokens, zero-width —
//!    and a required-but-absent end tag, which is the node-level hole
//!    [`ElementClose::Missing`](surface::ElementClose).
//! 2. **Bytes with no structural home at tree level are a typed
//!    `Unexpected` node** ([`SurfaceChild::Unexpected`](surface::SurfaceChild))
//!    carrying the verbatim slice: a stray end tag that matches no open
//!    element, an empty `</>`, trailing bytes the tokenizer consumed
//!    without reporting structure.
//! 3. **Bytes with no structural home inside a tag ride in the next
//!    token's [`leading`](surface::Token::leading) slice**, under the
//!    diagnostic the tokenizer already reported (a stray `/` in
//!    `<div / a>`, junk between an end-tag name and its `>`). In
//!    well-formed source `leading` is pure inter-token whitespace; it is
//!    verbatim source either way, so fidelity never depends on
//!    well-formedness. This is the one place S1 diverges from
//!    SwiftSyntax's `UnexpectedNodes`-between-any-two-children — recorded
//!    in the P2-7 record with the reasoning.
//!
//! # Strings and residency
//!
//! Every string in the tree is a `&'a str` slice of the input (P1-10: no
//! owned strings in nodes, ever). Containers are `vize_s0::{Box, Vec}`,
//! whose const assertion rejects `Drop` payloads, so the tree is Drop-free
//! and arena-resident by construction. Node footprints are pinned by
//! `#[cfg(target_pointer_width = "64")]`-guarded const asserts in
//! [`surface`].

#![no_std]

extern crate alloc;

pub mod parse;
pub mod render;
pub mod surface;

mod build;
mod event;

pub use parse::{SurfaceError, parse};
pub use render::{HoleCounts, check_fidelity, hole_counts, render};
pub use surface::{
    AttrValue, Attribute, CloseTag, Element, ElementClose, Interpolation, OpenTag, SurfaceChild,
    SurfaceTree, Token, TokenStatus,
};
