//! Parser for Art files (*.art.vue), on the Davinci S0/S1 stages (P4-13).
//!
//! An Art file is an SFC whose gallery definition is the `<art>` custom
//! block:
//!
//! 1. **S0 block splitting** ([`blocks`]) — the shared SFC container scan
//!    yields `<script>` / `<style>` blocks and the `<art>` custom block as
//!    S0 [`SourceBlock`](vize_s0::SourceBlock) frames over the authored file.
//! 2. **S1 tree** ([`art_block`]) — the `<art>` block parses into one
//!    lossless S1 surface tree; metadata comes from its open tag's attribute
//!    tokens and variants are its `<variant>` elements ([`variant`]).
//!
//! Every string in the descriptor is still a slice of the source (or an
//! arena string); offsets are file-absolute through the S0 frame.

mod art_block;
mod attrs;
mod blocks;
mod status;
mod variant;

#[cfg(test)]
mod divergence_tests;
#[cfg(test)]
mod golden_tests;

use crate::types::{ArtDescriptor, ArtParseError, ArtParseOptions, ArtParseResult, SourceLocation};
use vize_s0::Allocator;

/// Parse an Art file (*.art.vue) into an ArtDescriptor.
///
/// Uses arena allocation for all internal collections.
/// All string data is borrowed from the source - zero allocations for strings.
///
/// # Example
///
/// ```
/// use vize_s0::Allocator;
/// use vize_musea::parse::parse_art;
/// use vize_musea::types::ArtParseOptions;
///
/// let allocator = Allocator::new();
/// let source = r#"
/// <art title="Button" component="./Button.vue">
///   <variant name="Primary" default>
///     <Button>Click me</Button>
///   </variant>
/// </art>
/// "#;
///
/// let result = parse_art(&allocator, source, ArtParseOptions::default());
/// assert!(result.is_ok());
/// ```
pub fn parse_art<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: ArtParseOptions,
) -> ArtParseResult<'a> {
    // Allocate filename in arena if provided, otherwise use empty str
    let filename: &'a str = if options.filename.is_empty() {
        ""
    } else {
        allocator.alloc_str(&options.filename)
    };

    // S0: split the SFC container.
    let blocks = blocks::split_blocks(allocator, source)?;
    let frame = blocks.art.ok_or(ArtParseError::NoArtBlock)?;

    // S1: the `<art>` block as a lossless surface tree.
    let (tree, _surface_errors) = vize_s1::parse(allocator, frame.source());
    let art = art_block::art_element(&tree).ok_or(ArtParseError::NoArtBlock)?;

    let define_art = blocks
        .script_setup
        .as_ref()
        .and_then(|script| parse_define_art_metadata(allocator, script.content));

    // Metadata from `<art>` attributes, with the defineArt() fallback.
    let (metadata, _warnings) =
        art_block::parse_metadata(allocator, &art.open, define_art.as_ref(), filename)?;

    let variants = variant::parse_variants(allocator, &art.children, frame)?;

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

/// Unknown-`status` warnings of the `<art>` attributes, or an empty list if
/// the file has no `<art>` block or its metadata cannot be parsed.
pub(crate) fn art_status_warnings<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    filename: &str,
) -> vize_s0::Vec<'a, &'a str> {
    let empty = || vize_s0::Vec::new_in(&allocator);
    let Some(frame) = blocks::split_blocks(allocator, source)
        .ok()
        .and_then(|blocks| blocks.art)
    else {
        return empty();
    };
    let (tree, _surface_errors) = vize_s1::parse(allocator, frame.source());
    let Some(art) = art_block::art_element(&tree) else {
        return empty();
    };
    match art_block::parse_metadata(allocator, &art.open, None, filename) {
        Ok((_, warnings)) => warnings,
        Err(_) => empty(),
    }
}

#[derive(Debug)]
pub(crate) struct DefineArtMetadata<'a> {
    pub component_name: Option<&'a str>,
    pub component: Option<&'a str>,
    pub title: Option<&'a str>,
    pub description: Option<&'a str>,
    pub category: Option<&'a str>,
    pub tags: vize_s0::Vec<'a, &'a str>,
    pub status: Option<&'a str>,
    pub order: Option<u32>,
}

impl<'a> DefineArtMetadata<'a> {
    fn new(allocator: &'a Allocator) -> Self {
        Self {
            component_name: None,
            component: None,
            title: None,
            description: None,
            category: None,
            tags: vize_s0::Vec::new_in(&allocator),
            status: None,
            order: None,
        }
    }
}

fn parse_define_art_metadata<'a>(
    allocator: &'a Allocator,
    script: &'a str,
) -> Option<DefineArtMetadata<'a>> {
    // Cheap pre-check: defineArt() can only return Some if the literal token
    // "defineArt" appears in the source (it is a compiler macro recognized by
    // name and cannot be aliased), so skip the OXC parse otherwise.
    if !script.contains("defineArt") {
        return None;
    }

    // The defineArt-only reader: one parse, none of the binding/scope
    // analysis the full `parse_script_setup` runs for other consumers, and
    // pinned equal to it by croquis's own differential battery.
    let art = vize_croquis::script_parser::parse_define_art(script)?;
    Some(define_art_metadata(allocator, &art))
}

/// The descriptor-side copy of a `defineArt()` call's metadata.
pub(crate) fn define_art_metadata<'a>(
    allocator: &'a Allocator,
    art: &vize_croquis::macros::ArtDefinition,
) -> DefineArtMetadata<'a> {
    let mut meta = DefineArtMetadata::new(allocator);

    meta.component_name = Some(allocator.alloc_str(art.component_name.as_str()));
    meta.component = art
        .component_source
        .as_ref()
        .map(|source| allocator.alloc_str(source.as_str()));
    meta.title = art
        .title
        .as_ref()
        .map(|value| allocator.alloc_str(value.as_str()));
    meta.description = art
        .description
        .as_ref()
        .map(|value| allocator.alloc_str(value.as_str()));
    meta.category = art
        .category
        .as_ref()
        .map(|value| allocator.alloc_str(value.as_str()));
    meta.status = art
        .status
        .as_ref()
        .map(|value| allocator.alloc_str(value.as_str()));
    meta.order = art.order;
    for tag in &art.tags {
        meta.tags.push(allocator.alloc_str(tag.as_str()));
    }
    meta
}

/// Source location of `[start, end)`; the line and column are computed from
/// the file-absolute start offset (1-based line, 0-based byte column).
pub(crate) fn calculate_location_fast(source: &str, start: u32, end: u32) -> SourceLocation {
    let before = prefix(source, start);
    let line_start = before
        .iter()
        .rposition(|&byte| byte == b'\n')
        .map_or(0, |at| at + 1);
    let column = (before.len() - line_start) as u32;
    SourceLocation::new(start, end, line_of(source, start), column)
}

/// 1-based line of a file-absolute offset (`\n`-delimited, as the
/// descriptor has always counted).
pub(crate) fn line_of(source: &str, offset: u32) -> u32 {
    prefix(source, offset)
        .iter()
        .filter(|&&byte| byte == b'\n')
        .count() as u32
        + 1
}

fn prefix(source: &str, offset: u32) -> &[u8] {
    let bytes = source.as_bytes();
    &bytes[..(offset as usize).min(bytes.len())]
}

#[cfg(test)]
mod tests;
