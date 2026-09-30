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
}

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
