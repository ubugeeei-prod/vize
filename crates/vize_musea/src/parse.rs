//! Parser for Art files (*.art.vue).
//!
//! High-performance zero-copy parser using arena allocation.
//! All string data is borrowed directly from the source.

mod art_block;
mod variant;

use crate::types::{
    ArtDescriptor, ArtParseError, ArtParseOptions, ArtParseResult, ArtScriptBlock, ArtStyleBlock,
    SourceLocation,
};
use memchr::{memchr, memmem};
use vize_carton::Bump;

/// Result type for parsing SFC blocks: (script_setup, script, styles)
type SfcBlocksParseResult<'a> = Result<
    (
        Option<ArtScriptBlock<'a>>,
        Option<ArtScriptBlock<'a>>,
        vize_carton::Vec<'a, ArtStyleBlock<'a>>,
    ),
    ArtParseError,
>;

/// Parse an Art file (*.art.vue) into an ArtDescriptor.
///
/// Uses arena allocation for all internal collections.
/// All string data is borrowed from the source - zero allocations for strings.
///
/// # Example
///
/// ```
/// use vize_carton::Bump;
/// use vize_musea::parse::parse_art;
/// use vize_musea::types::ArtParseOptions;
///
/// let allocator = Bump::new();
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
#[inline]
pub fn parse_art<'a>(
    allocator: &'a Bump,
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

    // Parse metadata from <art> attributes
    let metadata = art_block::parse_metadata(allocator, &art_block)?;

    // Parse <variant> blocks inside <art>
    let variants = variant::parse_variants(
        allocator,
        art_block.content,
        source,
        art_block.content_start,
    )?;

    // Parse standard SFC blocks (script, style)
    let (script_setup, script, styles) = parse_sfc_blocks(allocator, source)?;

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

/// Internal representation of a found block.
#[derive(Debug)]
pub(crate) struct BlockInfo<'a> {
    /// Raw attributes string
    pub attrs_str: &'a str,
    /// Content between open and close tags
    pub content: &'a str,
    /// Byte offset where content starts (for line calculation)
    pub content_start: usize,
}

/// Parse SFC blocks (script, style) from source.
#[inline]
fn parse_sfc_blocks<'a>(allocator: &'a Bump, source: &'a str) -> SfcBlocksParseResult<'a> {
    let bytes = source.as_bytes();
    let mut script_setup: Option<ArtScriptBlock<'a>> = None;
    let mut script: Option<ArtScriptBlock<'a>> = None;
    let mut styles = vize_carton::Vec::new_in(allocator);

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
#[inline]
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
#[inline]
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

/// Extract an attribute value from an attributes string.
/// Uses byte-level operations for speed.
#[inline]
pub(crate) fn extract_attr<'a>(attrs: &'a str, name: &str) -> Option<&'a str> {
    let bytes = attrs.as_bytes();
    let name_bytes = name.as_bytes();

    // Search for name followed by '='
    let mut pos = 0;
    while pos < bytes.len() {
        // Find potential match
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
#[inline]
pub(crate) fn has_attr_fast(bytes: &[u8], name: &[u8]) -> bool {
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
#[inline]
pub(crate) fn has_attr(attrs: &str, name: &str) -> bool {
    has_attr_fast(attrs.as_bytes(), name.as_bytes())
}

/// Fast source location calculation.
/// Only counts lines up to start position (lazy end line computation).
#[inline]
pub(crate) fn calculate_location_fast(source: &str, start: u32, end: u32) -> SourceLocation {
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

#[cfg(test)]
mod tests {
    use super::{extract_attr, has_attr, parse_art};
    use crate::types::{ArtParseError, ArtParseOptions};
    use vize_carton::Bump;

    #[test]
    fn test_parse_simple_art() {
        let allocator = Bump::new();
        let source = r#"
<art title="Button" component="./Button.vue">
  <variant name="Primary" default>
    <Button>Click me</Button>
  </variant>
</art>

<script setup lang="ts">
import Button from './Button.vue'
</script>
"#;

        let result = parse_art(&allocator, source, ArtParseOptions::default());
        assert!(result.is_ok());

        let desc = result.unwrap();
        assert_eq!(desc.metadata.title, "Button");
        assert_eq!(desc.metadata.component, Some("./Button.vue"));
        assert_eq!(desc.variants.len(), 1);
        assert_eq!(desc.variants[0].name, "Primary");
        assert!(desc.variants[0].is_default);
        assert!(desc.script_setup.is_some());
    }

    #[test]
    fn test_parse_multiple_variants() {
        let allocator = Bump::new();
        let source = r#"
<art title="Button">
  <variant name="Primary" default>
    <Button variant="primary">Click</Button>
  </variant>
  <variant name="Secondary">
    <Button variant="secondary">Click</Button>
  </variant>
  <variant name="Disabled">
    <Button disabled>Click</Button>
  </variant>
</art>
"#;

        let result = parse_art(&allocator, source, ArtParseOptions::default());
        assert!(result.is_ok());

        let desc = result.unwrap();
        assert_eq!(desc.variants.len(), 3);
        assert_eq!(desc.variants[0].name, "Primary");
        assert_eq!(desc.variants[1].name, "Secondary");
        assert_eq!(desc.variants[2].name, "Disabled");
    }

    #[test]
    fn test_extract_attr() {
        assert_eq!(extract_attr(r#"title="Hello""#, "title"), Some("Hello"));
        assert_eq!(extract_attr(r#"title='Hello'"#, "title"), Some("Hello"));
        assert_eq!(extract_attr(r#"title=Hello"#, "title"), Some("Hello"));
        assert_eq!(extract_attr(r#"foo="bar""#, "title"), None);
    }

    #[test]
    fn test_has_attr() {
        assert!(has_attr("default scoped", "default"));
        assert!(has_attr("scoped default", "default"));
        assert!(has_attr("default", "default"));
        assert!(!has_attr("defaults", "default"));
    }

    #[test]
    fn test_missing_title_error() {
        let allocator = Bump::new();
        let source = r#"<art><variant name="Test"></variant></art>"#;
        let result = parse_art(&allocator, source, ArtParseOptions::default());
        assert!(matches!(result, Err(ArtParseError::MissingTitle)));
    }

    #[test]
    fn test_no_art_block_error() {
        let allocator = Bump::new();
        let source = r#"<template><div>Hello</div></template>"#;
        let result = parse_art(&allocator, source, ArtParseOptions::default());
        assert!(matches!(result, Err(ArtParseError::NoArtBlock)));
    }
}
