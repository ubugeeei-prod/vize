//! L1 container: the file-format layer (#6837).
//!
//! A container splits a file into its top-level blocks (an SFC's
//! `<template>`, `<script>`, `<style>` and custom blocks) without reading
//! inside them; each block's content is handed to the grammar or language
//! that owns it. The split is lossless: blocks are spans, and the bytes
//! between them stay in the source, so rendering the source is the identity.
//!
//! `vue` is the only format today. Svelte, Analog and TSRX containers join as
//! further [`ContainerFormat`] implementors with their own block rules.
//!
//! The opt-in stage-capture CLI uses this container's authored byte spans.
//! Full product SFC descriptor migration remains #6837.

pub mod vue;

use vize_l0::{Allocator, Span, Vec};

pub use vue::Vue;

/// A file format that splits into top-level blocks.
pub trait ContainerFormat {
    /// Split `source` into blocks, recording recoverable errors.
    fn split<'a>(&self, allocator: &'a Allocator, source: &'a str) -> Container<'a>;
}

/// The block structure of one file.
#[derive(Debug)]
pub struct Container<'a> {
    pub source: &'a str,
    /// Blocks in source order.
    pub blocks: Vec<'a, Block<'a>>,
    pub errors: Vec<'a, ContainerError>,
}

impl<'a> Container<'a> {
    /// Visit the complete authored file as alternating interstitial bytes and
    /// top-level blocks. Empty interstitial segments are retained, so the
    /// iterator always yields `blocks.len() * 2 + 1` segments without
    /// allocating.
    ///
    /// Concatenating each segment's [`ContainerSegment::span`] against
    /// [`Self::source`] reproduces the original file byte-for-byte. This is a
    /// structural view only; it does not grant descriptor or product-route
    /// admission.
    pub fn segments(&self) -> ContainerSegments<'_, 'a> {
        ContainerSegments {
            blocks: &self.blocks,
            source_end: u32::try_from(self.source.len()).unwrap_or(u32::MAX),
            index: 0,
        }
    }
}

/// One top-level block.
#[derive(Debug)]
pub struct Block<'a> {
    /// The tag name as written (`template`, `script`, `i18n`).
    pub name: &'a str,
    /// The whole open tag, `<` through `>`.
    pub open_tag: Span,
    pub attrs: Vec<'a, BlockAttr<'a>>,
    /// The raw content between the open and close tags.
    pub content: Span,
    /// The close tag, `</` through `>`; `None` when it is missing.
    pub close_tag: Option<Span>,
}

impl<'a> Block<'a> {
    /// The first attribute named `name` (`lang`, `setup`, `scoped`).
    pub fn attr(&self, name: &str) -> Option<&BlockAttr<'a>> {
        self.attrs.iter().find(|attr| attr.name == name)
    }

    /// The complete authored block, from the opening `<` through the closing
    /// `>`. A missing close tag extends through the captured content.
    pub const fn authored_span(&self) -> Span {
        Span::new(
            self.open_tag.start,
            match self.close_tag {
                Some(close) => close.end,
                None => self.content.end,
            },
        )
    }
}

/// One ordered piece of a lossless container capture.
#[derive(Debug)]
pub enum ContainerSegment<'container, 'source> {
    /// Bytes outside top-level blocks, including comments and whitespace.
    Interstitial(Span),
    /// One original top-level block.
    Block(&'container Block<'source>),
}

impl ContainerSegment<'_, '_> {
    /// The segment's file-absolute authored span.
    pub const fn span(&self) -> Span {
        match self {
            Self::Interstitial(span) => *span,
            Self::Block(block) => block.authored_span(),
        }
    }
}

/// Allocation-free iterator returned by [`Container::segments`].
#[derive(Debug)]
pub struct ContainerSegments<'container, 'source> {
    blocks: &'container [Block<'source>],
    source_end: u32,
    index: usize,
}

impl<'container, 'source> Iterator for ContainerSegments<'container, 'source> {
    type Item = ContainerSegment<'container, 'source>;

    fn next(&mut self) -> Option<Self::Item> {
        let len = self.blocks.len().checked_mul(2)?.checked_add(1)?;
        if self.index >= len {
            return None;
        }
        let index = self.index;
        self.index += 1;
        if index % 2 == 1 {
            return self.blocks.get(index / 2).map(ContainerSegment::Block);
        }

        let gap = index / 2;
        let start = gap
            .checked_sub(1)
            .and_then(|previous| self.blocks.get(previous))
            .map_or(0, |block| block.authored_span().end);
        let end = self
            .blocks
            .get(gap)
            .map_or(self.source_end, |block| block.open_tag.start);
        Some(ContainerSegment::Interstitial(Span::new(start, end)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len();
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for ContainerSegments<'_, '_> {
    fn len(&self) -> usize {
        self.blocks
            .len()
            .checked_mul(2)
            .and_then(|len| len.checked_add(1))
            .unwrap_or(usize::MAX)
            .saturating_sub(self.index)
    }
}

impl core::iter::FusedIterator for ContainerSegments<'_, '_> {}

/// An attribute on a block's open tag. Values are raw (quotes excluded,
/// entities undecoded).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockAttr<'a> {
    pub name: &'a str,
    pub value: Option<&'a str>,
    /// The whole attribute, name through closing quote.
    pub span: Span,
}

/// A recoverable container error at a byte offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainerError {
    pub code: ContainerErrorCode,
    pub offset: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerErrorCode {
    /// L1 spans cannot address a source larger than u32::MAX bytes.
    SourceTooLarge,
    /// An interpolation did not close.
    /// Consumers must not attribute the resulting block span to compilation.
    UncertainInterpolation,
    /// An open tag reaches the end of the file.
    UnterminatedOpenTag,
    /// A block has no close tag.
    MissingCloseTag,
    /// A second block where the format allows one (`<template>`,
    /// `<script>`, `<script setup>`).
    DuplicateBlock,
}
