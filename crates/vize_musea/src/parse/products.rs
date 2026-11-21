//! A single original container and Art tree serve both gallery queries.
use super::{art_block, blocks, descriptor_from_element};
use crate::{ArtParseError, ArtParseOptions, ArtParseResult};
use vize_l0::{Allocator, Vec};

/// The gallery's complete descriptor result and the public status warnings.
/// A variant or target inference error can coexist with a warning on the opening tag.
pub struct ArtSourceProducts<'a> {
    pub descriptor: ArtParseResult<'a>,
    pub status_warnings: Vec<'a, &'a str>,
}

/// Split the source once and read both products from the same lossless Art
/// element. The returned descriptor can drive the existing Vue preview, CSF,
/// Markdown and catalog generators with their own options.
pub fn parse_art_products<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: ArtParseOptions,
) -> ArtSourceProducts<'a> {
    let refused = |error| ArtSourceProducts {
        descriptor: Err(error),
        status_warnings: Vec::new_in(&allocator),
    };
    let blocks = match blocks::split_blocks(allocator, source) {
        Ok(blocks) => blocks,
        Err(error) => return refused(error),
    };
    let Some(frame) = blocks.art else {
        return refused(ArtParseError::NoArtBlock);
    };
    let (tree, _) = vize_l1::markup::parse(allocator, frame.source());
    let Some(art) = art_block::art_element(&tree) else {
        return refused(ArtParseError::NoArtBlock);
    };
    let filename = if options.filename.is_empty() {
        ""
    } else {
        allocator.alloc_str(&options.filename)
    };
    // The independently published warnings inspect tag attributes only. A macro
    // can supply the descriptor's status without producing a public warning.
    let status_warnings = art_block::parse_metadata(allocator, &art.open, None, filename)
        .map_or_else(|_| Vec::new_in(&allocator), |(_, warnings)| warnings);
    let descriptor = descriptor_from_element(allocator, source, filename, blocks, frame, art);
    ArtSourceProducts {
        descriptor,
        status_warnings,
    }
}
