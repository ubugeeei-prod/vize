//! A single original container and Art tree serve both gallery queries.
use super::{art_block, attrs, blocks, infer_art_target, parse_art_macro_metadata, variant};
use crate::{ArtDescriptor, ArtParseError, ArtParseOptions, ArtParseResult};
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
    let descriptor = read_descriptor(allocator, source, filename, blocks, frame, art);
    ArtSourceProducts {
        descriptor,
        status_warnings,
    }
}

fn read_descriptor<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    filename: &'a str,
    blocks: blocks::ArtBlocks<'a>,
    frame: vize_l0::SourceBlock<'a>,
    art: &vize_l1::markup::Element<'a>,
) -> ArtParseResult<'a> {
    let variants = variant::parse_variants(allocator, &art.children, frame)?;
    let mut art_metadata = blocks
        .script_setup
        .as_ref()
        .and_then(|script| parse_art_macro_metadata(allocator, script.content));
    if art_metadata
        .as_ref()
        .is_none_or(|metadata| metadata.component.is_none())
        && !blocks.is_inline
        && attrs::attr_value(&art.open, "component").is_none()
        && let Some(script) = blocks.script_setup.as_ref()
    {
        infer_art_target(allocator, script.content, &variants, &mut art_metadata)?;
    }

    // Metadata from `<art>` attributes, with the defineArt() fallback.
    let (metadata, _warnings) =
        art_block::parse_metadata(allocator, &art.open, art_metadata.as_ref(), filename)?;

    Ok(ArtDescriptor {
        filename,
        source,
        metadata,
        variants,
        script_setup: blocks.script_setup,
        script: blocks.script,
        styles: blocks.styles,
    })
}
