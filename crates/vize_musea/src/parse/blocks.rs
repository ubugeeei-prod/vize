//! S0 block splitting for Art files (Davinci P4-13).
//!
//! An `.art.vue` file is an SFC: `<art>` is a custom block beside the
//! ordinary `<script>` / `<style>` blocks. The split is the shared SFC
//! container scan (`vize_croquis::sfc::parse_sfc`, the same one compile,
//! lint and the LSP read), never a private substring search, and every
//! block this module hands on is an S0 [`SourceBlock`] frame over the
//! complete authored file, so each later slice keeps a file-absolute
//! offset by construction.

use std::borrow::Cow;

use vize_croquis::sfc::{
    BlockLocation, SfcError, SfcParseOptions, SfcScriptBlock, SfcStyleBlock, parse_sfc,
};
use vize_s0::{Allocator, SourceBlock, SourceRoot, ToCompactString, Vec};

use super::calculate_location_fast;
use crate::types::{ArtParseError, ArtScriptBlock, ArtStyleBlock};

/// The custom-block name that carries the gallery definition.
const ART_BLOCK: &str = "art";

/// The blocks an Art file consumer reads, split at S0.
pub(crate) struct ArtBlocks<'a> {
    /// The first `<art>` custom block's whole element extent
    /// (`<art …>` through `</art>`), framed against the full source.
    pub art: Option<SourceBlock<'a>>,
    pub script_setup: Option<ArtScriptBlock<'a>>,
    pub script: Option<ArtScriptBlock<'a>>,
    pub styles: Vec<'a, ArtStyleBlock<'a>>,
}

/// Split `source` into its SFC blocks.
///
/// A container-level failure (an unterminated or duplicated block) is the
/// SFC splitter's own diagnostic, surfaced as [`ArtParseError::ParseError`]
/// at the block's line.
pub(crate) fn split_blocks<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> Result<ArtBlocks<'a>, ArtParseError> {
    let root = SourceRoot::new(source).map_err(|_| ArtParseError::ParseError {
        line: 1,
        message: "Art source exceeds the u32 offset space".to_compact_string(),
    })?;
    let descriptor = parse_sfc(source, SfcParseOptions::default())
        .map_err(|error| container_error(source, error))?;

    let art = descriptor
        .custom_blocks
        .iter()
        .find(|block| block.block_type == ART_BLOCK)
        .map(|block| element_frame(root, source, &block.loc))
        .transpose()?;
    let mut styles = Vec::with_capacity_in(descriptor.styles.len(), &allocator);
    for style in &descriptor.styles {
        styles.push(style_block(allocator, source, style));
    }
    Ok(ArtBlocks {
        art,
        script_setup: descriptor
            .script_setup
            .as_ref()
            .map(|block| script_block(allocator, source, block)),
        script: descriptor
            .script
            .as_ref()
            .map(|block| script_block(allocator, source, block)),
        styles,
    })
}

/// Map a container-level failure. An `<art>` block the splitter cannot
/// close is no `<art>` block at all ([`ArtParseError::NoArtBlock`], the
/// long-standing contract for an unterminated `<art>`); any other block's
/// failure is reported with the splitter's own message at its line.
fn container_error(source: &str, error: SfcError) -> ArtParseError {
    let Some(loc) = error.loc else {
        return ArtParseError::ParseError {
            line: 1,
            message: error.message,
        };
    };
    if opens_art_block(source, loc.tag_start) {
        return ArtParseError::NoArtBlock;
    }
    ArtParseError::ParseError {
        line: saturating_u32(loc.start_line),
        message: error.message,
    }
}

/// Whether `<art` opens a block at `at` under the splitter's tag-name
/// alphabet (`[A-Za-z0-9_-]`, so `<article` and `<art-x` are other blocks).
fn opens_art_block(source: &str, at: usize) -> bool {
    let rest = source.as_bytes().get(at..).unwrap_or_default();
    rest.starts_with(b"<art")
        && !rest
            .get(4)
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

/// The S0 frame over a block's whole element extent.
fn element_frame<'a>(
    root: SourceRoot<'a>,
    source: &'a str,
    loc: &BlockLocation,
) -> Result<SourceBlock<'a>, ArtParseError> {
    let element = source
        .get(loc.tag_start..loc.tag_end)
        .ok_or(ArtParseError::NoArtBlock)?;
    root.block(element, saturating_u32(loc.tag_start))
        .map_err(|_| ArtParseError::NoArtBlock)
}

fn script_block<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    block: &SfcScriptBlock<'a>,
) -> ArtScriptBlock<'a> {
    ArtScriptBlock {
        content: content_slice(source, &block.loc).trim(),
        lang: block_attr(allocator, block.attrs.get("lang")),
        setup: block.setup,
        loc: Some(block_location(source, &block.loc)),
    }
}

fn style_block<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    block: &SfcStyleBlock<'a>,
) -> ArtStyleBlock<'a> {
    ArtStyleBlock {
        content: content_slice(source, &block.loc).trim(),
        lang: block_attr(allocator, block.attrs.get("lang")),
        scoped: block.scoped,
        loc: Some(block_location(source, &block.loc)),
    }
}

/// The block's content as a slice of the authored source (the splitter
/// reports byte offsets; slicing the root keeps the `'a` borrow).
fn content_slice<'a>(source: &'a str, loc: &BlockLocation) -> &'a str {
    source.get(loc.start..loc.end).unwrap_or_default()
}

/// A block attribute value with the source's `'a` borrow. The splitter
/// borrows every attribute value; an owned value (never produced today)
/// is interned into the arena rather than dropped.
fn block_attr<'a>(allocator: &'a Allocator, value: Option<&Cow<'a, str>>) -> Option<&'a str> {
    Some(match value? {
        Cow::Borrowed(value) => value,
        Cow::Owned(value) => allocator.alloc_str(value),
    })
}

/// The whole block's location: `<tag` through the end tag's `>`.
fn block_location(source: &str, loc: &BlockLocation) -> crate::types::SourceLocation {
    calculate_location_fast(
        source,
        saturating_u32(loc.tag_start),
        saturating_u32(loc.tag_end),
    )
}

fn saturating_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
