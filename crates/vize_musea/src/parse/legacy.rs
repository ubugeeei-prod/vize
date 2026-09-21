//! The pre-Davinci hand scanner, retained **only** as the P4-13
//! differential oracle (`differential_tests.rs`). Test-only: nothing in the
//! shipped crate can reach it, and it is deleted once the differential has
//! landed green (charter #26 — no dual lane survives the task).
//!
//! The scanning code below is the `origin/main` implementation verbatim
//! (`parse.rs` / `parse/art_block.rs` / `parse/variant.rs` before P4-13);
//! only the `defineArt()` reader, which never scanned blocks, is shared.

mod art_block;
mod blocks;
mod variant;

use super::{DefineArtMetadata, define_art_metadata};

/// The old `defineArt()` reader: the full script-setup analysis. Kept here
/// so the oracle stays independent of the defineArt-only reader under test.
fn parse_define_art_metadata<'a>(
    allocator: &'a Allocator,
    script: &'a str,
) -> Option<DefineArtMetadata<'a>> {
    memmem::find(script.as_bytes(), b"defineArt")?;
    let parsed = vize_croquis::script_parser::parse_script_setup(script);
    let art = parsed.macros.define_art()?;
    Some(define_art_metadata(allocator, art))
}
use crate::types::{ArtDescriptor, ArtParseOptions, ArtParseResult, SourceLocation};
use blocks::parse_sfc_blocks;
use memchr::{memchr, memmem};
use vize_s0::Allocator;

/// The old `parse_art`.
pub(crate) fn parse_art_legacy<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: ArtParseOptions,
) -> ArtParseResult<'a> {
    let bytes = source.as_bytes();

    // Allocate filename in arena if provided, otherwise use empty str
    let filename: &'a str = if options.filename.is_empty() {
        ""
    } else {
        allocator.alloc_str(&options.filename)
    };

    // Find <art> block using fast byte search
    let art_block = art_block::find_art_block(bytes, source)?;

    // Parse standard SFC blocks (script, style)
    let (script_setup, script, styles) = parse_sfc_blocks(allocator, source)?;
    let define_art = script_setup
        .as_ref()
        .and_then(|script| parse_define_art_metadata(allocator, script.content));

    // Parse metadata from <art> attributes and defineArt() fallback.
    let (metadata, _warnings) =
        art_block::parse_metadata(allocator, &art_block, define_art.as_ref(), filename)?;

    // Parse <variant> blocks inside <art>
    let variants = variant::parse_variants(
        allocator,
        art_block.content,
        source,
        art_block.content_start,
    )?;

    Ok(ArtDescriptor {
        filename,
        source,
        metadata,
        variants,
        script_setup,
        script,
        styles,
    })
}

/// The old `parse_art_status_warnings`.
pub(crate) fn parse_art_status_warnings_legacy<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    filename: &str,
) -> vize_s0::Vec<'a, &'a str> {
    let Ok(block) = art_block::find_art_block(source.as_bytes(), source) else {
        return vize_s0::Vec::new_in(&allocator);
    };
    match art_block::parse_metadata(allocator, &block, None, filename) {
        Ok((_, warnings)) => warnings,
        Err(_) => vize_s0::Vec::new_in(&allocator),
    }
}

/// Internal representation of a found block.
#[derive(Debug)]
struct BlockInfo<'a> {
    /// Raw attributes string
    attrs_str: &'a str,
    /// Content between open and close tags
    content: &'a str,
    /// Byte offset where content starts (for line calculation)
    content_start: usize,
}

/// Extract an attribute value from an attributes string.
fn extract_attr<'a>(attrs: &'a str, name: &str) -> Option<&'a str> {
    let bytes = attrs.as_bytes();
    let name_bytes = name.as_bytes();

    // Search for name followed by '='
    let mut pos = 0;
    while pos < bytes.len() {
        if let Some(offset) = memmem::find(&bytes[pos..], name_bytes) {
            let match_pos = pos + offset;
            let after_name = match_pos + name_bytes.len();

            // Check if followed by '='
            if after_name < bytes.len() && bytes[after_name] == b'=' {
                // Check word boundary before
                let before_ok = match_pos == 0 || bytes[match_pos - 1].is_ascii_whitespace();

                if before_ok {
                    let value_start = after_name + 1;
                    if value_start >= bytes.len() {
                        return None;
                    }

                    // Check for quoted value
                    let quote = bytes[value_start];
                    if quote == b'"' || quote == b'\'' {
                        let search_start = value_start + 1;
                        if let Some(end_offset) = memchr(quote, &bytes[search_start..]) {
                            return Some(&attrs[search_start..search_start + end_offset]);
                        }
                    } else {
                        // Unquoted value - find end
                        let mut end = value_start;
                        while end < bytes.len()
                            && !bytes[end].is_ascii_whitespace()
                            && bytes[end] != b'>'
                            && bytes[end] != b'/'
                        {
                            end += 1;
                        }
                        if end > value_start {
                            return Some(&attrs[value_start..end]);
                        }
                    }
                }
            }
            pos = match_pos + 1;
        } else {
            break;
        }
    }

    None
}

/// Fast boolean attribute check using byte operations.
fn has_attr_fast(bytes: &[u8], name: &[u8]) -> bool {
    let mut pos = 0;
    while pos < bytes.len() {
        if let Some(offset) = memmem::find(&bytes[pos..], name) {
            let match_pos = pos + offset;
            let after_name = match_pos + name.len();

            // Check word boundaries
            let before_ok = match_pos == 0 || bytes[match_pos - 1].is_ascii_whitespace();
            let after_ok = after_name >= bytes.len()
                || bytes[after_name].is_ascii_whitespace()
                || bytes[after_name] == b'>'
                || bytes[after_name] == b'='
                || bytes[after_name] == b'/';

            if before_ok && after_ok {
                return true;
            }
            pos = match_pos + 1;
        } else {
            break;
        }
    }
    false
}

/// Check if an attribute is present (boolean attribute).
fn has_attr(attrs: &str, name: &str) -> bool {
    has_attr_fast(attrs.as_bytes(), name.as_bytes())
}

/// The old location helper (identical to the retained
/// [`super::calculate_location_fast`]; kept here so the oracle shares no
/// scanning-side code with the implementation under test).
fn calculate_location_fast(source: &str, start: u32, end: u32) -> SourceLocation {
    let bytes = source.as_bytes();
    let start_usize = start as usize;

    // Count newlines before start using memchr iterator
    let mut line = 1u32;
    let mut last_newline = 0usize;

    for pos in memchr::memchr_iter(b'\n', &bytes[..start_usize]) {
        line += 1;
        last_newline = pos + 1;
    }

    let column = (start_usize - last_newline) as u32;

    SourceLocation::new(start, end, line, column)
}
