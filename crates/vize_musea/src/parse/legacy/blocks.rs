//! The old `<script>` / `<style>` substring scan (P4-13 differential
//! oracle; verbatim).

use super::calculate_location_fast;
use super::{extract_attr, has_attr_fast};
use crate::types::{ArtParseError, ArtScriptBlock, ArtStyleBlock};
use memchr::{memchr, memmem};
use vize_s0::Allocator;

/// Result type for parsing SFC blocks: (script_setup, script, styles)
type SfcBlocksParseResult<'a> = Result<
    (
        Option<ArtScriptBlock<'a>>,
        Option<ArtScriptBlock<'a>>,
        vize_s0::Vec<'a, ArtStyleBlock<'a>>,
    ),
    ArtParseError,
>;

/// Parse SFC blocks (script, style) from source.
pub(super) fn parse_sfc_blocks<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> SfcBlocksParseResult<'a> {
    let bytes = source.as_bytes();
    let mut script_setup: Option<ArtScriptBlock<'a>> = None;
    let mut script: Option<ArtScriptBlock<'a>> = None;
    let mut styles = vize_s0::Vec::new_in(&allocator);

    // Use memmem finder for repeated searches (amortized O(n))
    let script_finder = memmem::Finder::new(b"<script");
    let style_finder = memmem::Finder::new(b"<style");

    let mut pos = 0;
    while pos < bytes.len() {
        // Find next script or style tag
        let script_pos = script_finder.find(&bytes[pos..]).map(|p| pos + p);
        let style_pos = style_finder.find(&bytes[pos..]).map(|p| pos + p);

        match (script_pos, style_pos) {
            (Some(sp), Some(stp)) if sp < stp => {
                if let Some((block, end)) = parse_script_block(source, sp)? {
                    if block.setup {
                        script_setup = Some(block);
                    } else {
                        script = Some(block);
                    }
                    pos = end;
                } else {
                    pos = sp + 1;
                }
            }
            (Some(sp), Some(stp)) if stp < sp => {
                if let Some((block, end)) = parse_style_block(source, stp)? {
                    styles.push(block);
                    pos = end;
                } else {
                    pos = stp + 1;
                }
            }
            (Some(sp), None) => {
                if let Some((block, end)) = parse_script_block(source, sp)? {
                    if block.setup {
                        script_setup = Some(block);
                    } else {
                        script = Some(block);
                    }
                    pos = end;
                } else {
                    pos = sp + 1;
                }
            }
            (None, Some(stp)) => {
                if let Some((block, end)) = parse_style_block(source, stp)? {
                    styles.push(block);
                    pos = end;
                } else {
                    pos = stp + 1;
                }
            }
            (None, None) => break,
            _ => pos += 1,
        }
    }

    Ok((script_setup, script, styles))
}

/// Parse a script block starting at `start`.
fn parse_script_block<'a>(
    source: &'a str,
    start: usize,
) -> Result<Option<(ArtScriptBlock<'a>, usize)>, ArtParseError> {
    let bytes = source.as_bytes();

    // Find '>' that closes the opening tag
    let Some(tag_end) = memchr(b'>', &bytes[start..]) else {
        return Ok(None);
    };
    let tag_end = start + tag_end;

    // Check for self-closing
    if bytes[tag_end - 1] == b'/' {
        return Ok(None);
    }

    // Parse attributes (skip "<script")
    let attrs_str = &source[start + 7..tag_end];
    let lang = extract_attr(attrs_str, "lang");
    let is_setup = has_attr_fast(attrs_str.as_bytes(), b"setup");

    // Find </script> using fast search
    let content_start = tag_end + 1;
    let close_finder = memmem::Finder::new(b"</script>");
    let Some(close_offset) = close_finder.find(&bytes[content_start..]) else {
        return Ok(None);
    };
    let close_pos = content_start + close_offset;

    let content = &source[content_start..close_pos];
    let loc = calculate_location_fast(source, start as u32, (close_pos + 9) as u32);

    Ok(Some((
        ArtScriptBlock {
            content: content.trim(),
            lang,
            setup: is_setup,
            loc: Some(loc),
        },
        close_pos + 9, // "</script>".len()
    )))
}

/// Parse a style block starting at `start`.
fn parse_style_block<'a>(
    source: &'a str,
    start: usize,
) -> Result<Option<(ArtStyleBlock<'a>, usize)>, ArtParseError> {
    let bytes = source.as_bytes();

    // Find '>' that closes the opening tag
    let Some(tag_end) = memchr(b'>', &bytes[start..]) else {
        return Ok(None);
    };
    let tag_end = start + tag_end;

    // Check for self-closing
    if bytes[tag_end - 1] == b'/' {
        return Ok(None);
    }

    // Parse attributes (skip "<style")
    let attrs_str = &source[start + 6..tag_end];
    let lang = extract_attr(attrs_str, "lang");
    let is_scoped = has_attr_fast(attrs_str.as_bytes(), b"scoped");

    // Find </style> using fast search
    let content_start = tag_end + 1;
    let close_finder = memmem::Finder::new(b"</style>");
    let Some(close_offset) = close_finder.find(&bytes[content_start..]) else {
        return Ok(None);
    };
    let close_pos = content_start + close_offset;

    let content = &source[content_start..close_pos];
    let loc = calculate_location_fast(source, start as u32, (close_pos + 8) as u32);

    Ok(Some((
        ArtStyleBlock {
            content: content.trim(),
            lang,
            scoped: is_scoped,
            loc: Some(loc),
        },
        close_pos + 8, // "</style>".len()
    )))
}
