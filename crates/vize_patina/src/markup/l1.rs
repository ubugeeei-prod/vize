//! The L1 (surface) backend of the markup facade for Vue templates.
//!
//! [`L1Markup`] views one L1 parse: the repaired tree the facade walks and
//! the non-repairing authored projection content-model rules read. Text,
//! `v-if` chains, `v-for` loops, `v-pre` subtrees and unwrapped `<template>`
//! carriers are the lowering's own decisions over L1
//! (`vize_l1_to_l2::lower::{TextLane, if_chain, structure}`), so this view
//! answers what the L2 view answers without lowering.
//!
//! The one structural difference: L2 inserts implicit `tbody` / `tr` table
//! owners; L1 keeps the authored table children.

use vize_l0::{Allocator, SourceRoot};
use vize_l1::markup::{SurfaceError, SurfaceTree};
use vize_l1_to_l2::lower::{TextLane, TextSource};
use vize_l2::op::Namespace;

pub(super) mod children;

/// A parsed Vue template the markup facade views at L1.
pub struct L1Markup<'a> {
    pub(super) source: &'a str,
    pub(super) allocator: &'a Allocator,
    pub(super) text: TextSource<'a>,
    /// The repaired surface tree, the one the lowering reads.
    pub(super) tree: &'a SurfaceTree<'a>,
    /// The non-repairing authored projection (`tree` when nothing was repaired).
    pub(super) authored: &'a SurfaceTree<'a>,
    pub(super) errors: &'a [SurfaceError],
}

impl<'a> L1Markup<'a> {
    /// Parse `source` once into the arena.
    ///
    /// `None` only for a source past `u32` offsets, which L1 parses to one
    /// unexpected hole: no element, text or interpolation for a rule to see.
    pub fn parse(allocator: &'a Allocator, source: &'a str) -> Option<&'a Self> {
        let root = SourceRoot::new(source).ok()?;
        let (tree, authored, errors) = vize_l1::markup::parse_with_authored(allocator, source);
        let tree: &'a SurfaceTree<'a> = allocator.alloc(tree);
        let authored: &'a SurfaceTree<'a> = match authored {
            Some(authored) => allocator.alloc(authored),
            None => tree,
        };
        let errors: &'a [SurfaceError] = allocator.alloc(errors);
        Some(allocator.alloc(Self {
            source,
            allocator,
            text: TextSource::new(allocator, root.whole_block()),
            tree,
            authored,
            errors,
        }))
    }

    /// The repaired surface tree.
    pub fn tree(&self) -> &'a SurfaceTree<'a> {
        self.tree
    }

    /// The tokenizer errors the parse reported.
    pub fn errors(&self) -> &'a [SurfaceError] {
        self.errors
    }

    /// The child-list context of the template's top level: comments are
    /// children, as the lint parse keeps them.
    pub(super) fn root_list(&self) -> children::ListCx {
        children::ListCx {
            lane: TextLane::root(true),
            frozen: false,
            ns: Namespace::Html,
        }
    }

    /// Rendered text, entity-decoded with the L2 decoder when it carries an
    /// entity.
    pub(super) fn decode_text(&self, text: &'a str) -> &'a str {
        decode_text(self.allocator, text)
    }

    /// An authored static attribute value, entity-decoded with the
    /// tokenizer's attribute-context rules.
    pub(super) fn decode_attribute(&self, value: &'a str) -> &'a str {
        decode_attribute(self.allocator, value)
    }

    pub(super) fn frozen_attribute_name(&self, name: &'a str) -> &'a str {
        vize_l1_to_l2::lower::frozen_attribute_name(self.allocator, name)
    }
}

/// Rendered text, entity-decoded with the L2 decoder when it carries an
/// entity (the one L2 decoder, never a second reading).
pub(super) fn decode_text<'a>(allocator: &'a Allocator, text: &'a str) -> &'a str {
    if text.contains('&') {
        allocator.alloc_str(vize_l1_to_l2::emit::decode_html_entities(text).as_str())
    } else {
        text
    }
}

/// An authored static attribute value, entity-decoded with the
/// tokenizer's attribute-context rules.
pub(super) fn decode_attribute<'a>(allocator: &'a Allocator, value: &'a str) -> &'a str {
    if value.contains('&') {
        allocator.alloc_str(vize_l1_to_l2::emit::decode_html_attribute_entities(value).as_str())
    } else {
        value
    }
}
