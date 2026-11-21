//! Parser for Art files (*.art.vue), on the Davinci L0/L1 stages (P4-13).
//!
//! An Art file is an SFC whose gallery definition is the `<art>` custom
//! block:
//!
//! 1. **L0 block splitting** ([`blocks`]) — the shared SFC container scan
//!    yields `<script>` / `<style>` blocks and the `<art>` custom block as
//!    L0 [`SourceBlock`](vize_l0::SourceBlock) frames over the authored file.
//! 2. **L1 tree** ([`art_block`]) — the `<art>` block parses into one
//!    lossless L1 surface tree; metadata comes from its open tag's attribute
//!    tokens and variants are its `<variant>` elements ([`variant`]).
//!
//! Every string in the descriptor is still a slice of the source (or an
//! arena string); offsets are file-absolute through the L0 frame.

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
use vize_l0::Allocator;

/// Parse an Art file (*.art.vue) into an ArtDescriptor.
///
/// Uses arena allocation for all internal collections.
/// All string data is borrowed from the source - zero allocations for strings.
///
/// # Example
///
/// ```
/// use vize_l0::Allocator;
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

    // L0: split the SFC container.
    let blocks = blocks::split_blocks(allocator, source)?;
    let frame = blocks.art.ok_or(ArtParseError::NoArtBlock)?;

    // L1: the `<art>` block as a lossless surface tree.
    let (tree, _surface_errors) = vize_l1::markup::parse(allocator, frame.source());
    let art = art_block::art_element(&tree).ok_or(ArtParseError::NoArtBlock)?;

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

fn infer_art_target<'a>(
    allocator: &'a Allocator,
    script: &'a str,
    variants: &[crate::types::ArtVariant<'a>],
    metadata: &mut Option<DefineArtMetadata<'a>>,
) -> Result<(), ArtParseError> {
    if variants.is_empty() {
        return Ok(());
    }
    let parsed = vize_croquis::script_parser::parse_script_setup(script);
    let shared = parsed
        .import_sources
        .iter()
        .filter(|(name, _)| {
            variants
                .iter()
                .all(|variant| template_uses_component(variant.template, name.as_str()))
        })
        .collect::<std::vec::Vec<_>>();
    let candidates = if shared.is_empty() {
        parsed
            .import_sources
            .iter()
            .filter(|(name, _)| {
                variants
                    .iter()
                    .any(|variant| template_uses_component(variant.template, name.as_str()))
            })
            .collect::<std::vec::Vec<_>>()
    } else {
        shared
    };
    if candidates.len() > 1 {
        let mut names = candidates
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<std::vec::Vec<_>>();
        names.sort_unstable();
        return Err(ArtParseError::AmbiguousTarget {
            candidates: names.join(", ").into(),
        });
    }
    let Some((name, source)) = candidates.first().copied() else {
        return Ok(());
    };
    let metadata = metadata.get_or_insert_with(|| DefineArtMetadata::new(allocator));
    metadata.component_name = Some(allocator.alloc_str(name.as_str()));
    metadata.component = Some(allocator.alloc_str(source.as_str()));
    Ok(())
}

fn template_uses_component(template: &str, name: &str) -> bool {
    let mut cursor = 0;
    while let Some(relative) = template.get(cursor..).and_then(|rest| rest.find('<')) {
        let start = cursor + relative + 1;
        let Some(rest) = template.get(start..) else {
            return false;
        };
        if rest.starts_with(name)
            && rest
                .as_bytes()
                .get(name.len())
                .is_some_and(|byte| byte.is_ascii_whitespace() || matches!(byte, b'/' | b'>'))
        {
            return true;
        }
        cursor = start;
    }
    false
}

/// Unknown-`status` warnings of the `<art>` attributes, or an empty list if
/// the file has no `<art>` block or its metadata cannot be parsed.
pub(crate) fn art_status_warnings<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    filename: &str,
) -> vize_l0::Vec<'a, &'a str> {
    let empty = || vize_l0::Vec::new_in(&allocator);
    let Some(frame) = blocks::split_blocks(allocator, source)
        .ok()
        .and_then(|blocks| blocks.art)
    else {
        return empty();
    };
    let (tree, _surface_errors) = vize_l1::markup::parse(allocator, frame.source());
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
    pub tags: vize_l0::Vec<'a, &'a str>,
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
            tags: vize_l0::Vec::new_in(&allocator),
            status: None,
            order: None,
        }
    }
}

fn parse_art_macro_metadata<'a>(
    allocator: &'a Allocator,
    script: &'a str,
) -> Option<DefineArtMetadata<'a>> {
    // Cheap pre-check: defineArt() can only return Some if the literal token
    // "defineArt" appears in the source (it is a compiler macro recognized by
    // name and cannot be aliased), so skip the OXC parse otherwise.
    let has_define_art = contains_identifier(script, "defineArt");
    let has_define_art_meta = contains_identifier(script, "defineArtMeta");
    if !has_define_art && !has_define_art_meta {
        return None;
    }

    // The defineArt-only reader: one parse, none of the binding/scope
    // analysis the full `parse_script_setup` runs for other consumers, and
    // pinned equal to it by croquis's own differential battery.
    let mut metadata = has_define_art
        .then(|| vize_croquis::script_parser::parse_define_art(script))
        .flatten()
        .as_ref()
        .map(|art| define_art_metadata(allocator, art))
        .unwrap_or_else(|| DefineArtMetadata::new(allocator));
    let found_define_art = metadata.component_name.is_some() || metadata.component.is_some();
    if let Some(meta) = has_define_art_meta
        .then(|| vize_croquis::script_parser::parse_define_art_meta(script))
        .flatten()
    {
        apply_art_metadata(allocator, &mut metadata, &meta);
        return Some(metadata);
    }
    found_define_art.then_some(metadata)
}

fn contains_identifier(source: &str, name: &str) -> bool {
    source.match_indices(name).any(|(start, _)| {
        let bytes = source.as_bytes();
        let identifier = |byte: u8| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$');
        !start
            .checked_sub(1)
            .and_then(|before| bytes.get(before))
            .is_some_and(|byte| identifier(*byte))
            && !bytes
                .get(start + name.len())
                .is_some_and(|byte| identifier(*byte))
    })
}

/// The descriptor-side copy of a `defineArt()` call's metadata.
pub(crate) fn define_art_metadata<'a>(
    allocator: &'a Allocator,
    art: &vize_croquis::macros::ArtDefinition,
) -> DefineArtMetadata<'a> {
    let mut meta = DefineArtMetadata::new(allocator);

    meta.component_name =
        (!art.component_name.is_empty()).then(|| allocator.alloc_str(art.component_name.as_str()));
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

fn apply_art_metadata<'a>(
    allocator: &'a Allocator,
    meta: &mut DefineArtMetadata<'a>,
    art: &vize_croquis::macros::ArtDefinition,
) {
    if let Some(value) = &art.title {
        meta.title = Some(allocator.alloc_str(value.as_str()));
    }
    if let Some(value) = &art.description {
        meta.description = Some(allocator.alloc_str(value.as_str()));
    }
    if let Some(value) = &art.category {
        meta.category = Some(allocator.alloc_str(value.as_str()));
    }
    if let Some(value) = &art.status {
        meta.status = Some(allocator.alloc_str(value.as_str()));
    }
    if let Some(order) = art.order {
        meta.order = Some(order);
    }
    if !art.tags.is_empty() {
        meta.tags.clear();
        for tag in &art.tags {
            meta.tags.push(allocator.alloc_str(tag.as_str()));
        }
    }
}

/// Source location of `[start, end)`; the line and column are computed from
/// the file-absolute start offset (1-based line, 0-based byte column).
pub(crate) fn calculate_location_fast(source: &str, start: u32, end: u32) -> SourceLocation {
    calculate_location_with_line(source, start, end, line_of(source, start))
}

/// Build a location when the caller already counted its starting line.
pub(crate) fn calculate_location_with_line(
    source: &str,
    start: u32,
    end: u32,
    line: u32,
) -> SourceLocation {
    let before = prefix(source, start);
    let line_start = before
        .iter()
        .rposition(|&byte| byte == b'\n')
        .map_or(0, |at| at + 1);
    let column = (before.len() - line_start) as u32;
    SourceLocation::new(start, end, line, column)
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
    bytes.get(..offset as usize).unwrap_or(bytes)
}

#[cfg(test)]
mod tests;

#[cfg(feature = "art-source-products")]
pub(crate) mod products;
