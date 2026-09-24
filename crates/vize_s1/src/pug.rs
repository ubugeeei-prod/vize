//! The lossless **pug** surface tree — charter #12's second S1 dialect
//! (Davinci P4-12c).
//!
//! [`parse_pug`] builds a typed tree whose in-order render is the exact
//! authored bytes, malformed input included, with the debug verifier
//! asserting `render(tree) == source` on every construction (TS-19's
//! law, the same contract as the Vue surface in [`crate::parse`]).
//!
//! # What "pug" means here
//!
//! The pinned `pug@3.0.4` exactly as Vue feeds it: the template content
//! is first dedented the way `@vue/compiler-sfc` dedents `lang="pug"`
//! blocks, then lexed and parsed by ports of `pug-lexer@5.0.1` and
//! `pug-parser@6.0.0` ([`lex`], [`build`]) — so every structural decision
//! (indentation, text-block extents, `#[…]` bounds, the newline between
//! piped lines) is pug's own. The Vue lowering
//! (`vize_s1_to_s2::lower::pug`) replays pug's code generator over this
//! tree; constructs that execute JavaScript at build time or link other
//! files (mixins, includes, conditionals, iteration, filters, code) are
//! recognised here with pug's patterns, kept verbatim as
//! [`PugNode::Refused`], and refused there with diagnostics.
//!
//! # The hole policy (pug)
//!
//! The Vue surface's three clauses, applied to pug:
//!
//! 1. **`Missing`**: an unterminated `#[` tag interpolation's `]` is a
//!    zero-width `Missing` token at the line end.
//! 2. **`Unexpected`**: bytes pug would throw on — an unlexable line
//!    rest, an invalid `.class`/`#id`, an unclosed `(`, an attribute
//!    error, a token pug's parser rejects, an indented block no construct
//!    owns — become [`PugNode::Unexpected`] nodes,
//!    [`PugTextPiece::Unexpected`] pieces or [`PugAttrItem::Unexpected`]
//!    items, each under a recorded [`PugError`].
//! 3. **`leading`** carries inter-token whitespace, the dedent prefix and
//!    `\r` bytes the lexer view drops, and pug's *trivia*: unbuffered
//!    `//-` comments with their bodies, which pug strips before parsing
//!    (`pug-strip-comments`) and which therefore never shape the tree.

mod build;
mod chars;
mod error;
mod lex;
mod logical;
mod render;
mod tree;

use vize_s0::{Allocator, Vec};

pub use error::{PugError, PugErrorCode};
pub use render::{PugHoleCounts, check_pug_fidelity, pug_hole_counts, render_pug};
pub use tree::{
    PugAttr, PugAttrGroup, PugAttrItem, PugBlock, PugCode, PugComment, PugDotBlock, PugHtml,
    PugHtmlItem, PugInline, PugInterpolation, PugNode, PugRefusal, PugRefused, PugTag, PugTagPart,
    PugText, PugTextPiece, PugTree, PugUnexpected,
};

/// Parse pug template content (as it sits between the SFC's `<template
/// lang="pug">` tags) into its lossless surface tree.
///
/// Total over arbitrary input: every byte lands in a token, malformed
/// source becomes typed holes plus [`PugError`]s, never a panic.
pub fn parse_pug<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> (PugTree<'a>, Vec<'a, PugError>) {
    // S1 addresses sources with `u32` offsets. A larger source keeps byte
    // fidelity as the end-of-file token's leading, with no nodes.
    if u32::try_from(source.len()).is_err() {
        let eof = crate::surface::Token::present(source, crate::slice::from(source, source.len()));
        let tree = PugTree {
            source,
            nodes: Vec::new_in(&allocator),
            eof,
        };
        return (tree, Vec::new_in(&allocator));
    }
    let logical = logical::Logical::new(allocator, source);
    let mut tokens = Vec::new_in(&allocator);
    let mut lex_errors = Vec::new_in(&allocator);
    lex::lex(allocator, logical.text, &mut tokens, &mut lex_errors);
    let mut errors = Vec::new_in(&allocator);
    for &(code, at) in lex_errors.iter() {
        errors.push(PugError {
            code,
            offset: logical.at(at as usize),
        });
    }
    let tree = build::build(allocator, source, &logical, &tokens, &mut errors);
    errors.sort_by_key(|error| error.offset);
    debug_assert!(
        check_pug_fidelity(&tree).is_ok(),
        "pug S1 fidelity: render(tree) != source"
    );
    (tree, errors)
}
