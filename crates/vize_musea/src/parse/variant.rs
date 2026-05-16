//! Parser for `<variant>` blocks.
//!
//! High-performance parser using arena allocation and zero-copy parsing.

use super::{calculate_location_fast, extract_attr, has_attr};
use crate::types::{ArtParseError, ArtVariant, ViewportConfig};
use memchr::{memchr, memmem};
use vize_carton::{Bump, FxHashMap, ToCompactString};

/// Parse all `<variant>` blocks from art content.
/// Uses arena allocation for the variants vector.
#[inline]
pub(crate) fn parse_variants<'a>(
    allocator: &'a Bump,
    content: &'a str,
    full_source: &'a str,
    content_offset: usize,
) -> Result<vize_carton::Vec<'a, ArtVariant<'a>>, ArtParseError> {
    let bytes = content.as_bytes();
    let mut variants = vize_carton::Vec::new_in(allocator);
    let mut pos = 0;

    // Use memmem finder for repeated <variant searches
    let variant_finder = memmem::Finder::new(b"<variant");

    while pos < bytes.len() {
        // Find next <variant using fast byte search
        let Some(start_offset) = variant_finder.find(&bytes[pos..]) else {
            break;
        };
        let start = pos + start_offset;

        // Verify it's actually <variant and not <variants etc
        let after_variant = start + 8;
        if after_variant < bytes.len() {
            let next_byte = bytes[after_variant];
            if next_byte == b' ' || next_byte == b'>' || next_byte == b'\n' || next_byte == b'\t' {
                let (variant, end) =
                    parse_single_variant(allocator, content, start, full_source, content_offset)?;
                variants.push(variant);
                pos = end;
                continue;
            }
        } else if after_variant == bytes.len() {
            // <variant at end of content - invalid but skip
            break;
        }

        pos = start + 1;
    }

    Ok(variants)
}

/// Parse a single `<variant>` block.
/// All strings are borrowed from source - zero allocations except for args JSON.
#[inline]
fn parse_single_variant<'a>(
    allocator: &'a Bump,
    content: &'a str,
    start: usize,
    full_source: &'a str,
    content_offset: usize,
) -> Result<(ArtVariant<'a>, usize), ArtParseError> {
    let bytes = content.as_bytes();
    let absolute_start = content_offset + start;
    let line = count_lines_fast(full_source.as_bytes(), absolute_start);

    // Find the closing '>' of the opening tag using fast byte search
    let Some(tag_end_rel) = memchr(b'>', &bytes[start..]) else {
        return Err(ArtParseError::ParseError {
            line,
            message: "Unclosed <variant> tag".to_compact_string(),
        });
    };
    let tag_end = start + tag_end_rel;

    // Extract attributes - zero copy slice
    let attrs_start = start + 8; // Skip "<variant"
    let attrs_str = content[attrs_start..tag_end].trim();

    // Parse name (required) - zero copy
    let name = extract_attr(attrs_str, "name").ok_or(ArtParseError::MissingVariantName { line })?;

    // Parse optional boolean attributes - fast byte comparison
    let is_default = has_attr(attrs_str, "default");
    let skip_vrt = has_attr(attrs_str, "skip-vrt") || has_attr(attrs_str, "skipVrt");

    // Parse args (JSON) - this may allocate for decoded strings
    let args = extract_attr(attrs_str, "args")
        .and_then(|s| parse_args_json(allocator, s).ok())
        .unwrap_or_default();

    // Parse viewport
    let viewport = parse_viewport(attrs_str);

    // Find </variant> using fast byte search
    let template_start = tag_end + 1;
    let close_finder = memmem::Finder::new(b"</variant>");

    let Some(close_pos_rel) = close_finder.find(&bytes[template_start..]) else {
        return Err(ArtParseError::ParseError {
            line,
            message: "Missing </variant> closing tag".to_compact_string(),
        });
    };
    let close_pos = template_start + close_pos_rel;

    // Extract and trim template content - zero copy
    let template = content[template_start..close_pos].trim();

    // Calculate location in full source
    let absolute_end = content_offset + close_pos + 10; // "</variant>".len()
    let loc = calculate_location_fast(full_source, absolute_start as u32, absolute_end as u32);

    Ok((
        ArtVariant {
            name,
            template,
            is_default,
            args,
            viewport,
            skip_vrt,
            loc: Some(loc),
        },
        close_pos + 10, // "</variant>".len()
    ))
}

/// Parse args JSON string into a map with arena-allocated keys.
/// HTML entities are decoded before parsing.
#[inline]
fn parse_args_json<'a>(
    allocator: &'a Bump,
    s: &str,
) -> Result<FxHashMap<&'a str, serde_json::Value>, serde_json::Error> {
    // Check if we need to decode HTML entities
    let needs_decode = s.contains('&');

    let json_str: std::borrow::Cow<'_, str> = if needs_decode {
        // Decode common HTML entities - allocates only when needed
        std::borrow::Cow::Owned(
            s.replace("&quot;", "\"")
                .replace("&apos;", "'")
                .replace("&lt;", "<")
                .replace("&gt;", ">")
                .replace("&amp;", "&"),
        )
    } else {
        std::borrow::Cow::Borrowed(s)
    };

    #[allow(clippy::disallowed_types)]
    let map: FxHashMap<std::string::String, serde_json::Value> = serde_json::from_str(&json_str)?;

    // Allocate keys in arena for zero-copy storage
    Ok(map
        .into_iter()
        .map(|(k, v)| {
            let key: &'a str = allocator.alloc_str(&k);
            (key, v)
        })
        .collect())
}

/// Parse viewport configuration from attributes.
/// Supports JSON format and simple "WxH" or "WxH@scale" format.
#[inline]
fn parse_viewport(attrs: &str) -> Option<ViewportConfig> {
    let viewport_str = extract_attr(attrs, "viewport")?;
    let bytes = viewport_str.as_bytes();

    // Try JSON format first: viewport='{"width":375,"height":667}'
    if !bytes.is_empty() && bytes[0] == b'{' {
        // Check if we need HTML entity decoding
        let json_str = if viewport_str.contains('&') {
            std::borrow::Cow::Owned(viewport_str.replace("&quot;", "\"").replace("&apos;", "'"))
        } else {
            std::borrow::Cow::Borrowed(viewport_str)
        };

        if let Ok(config) = serde_json::from_str::<ViewportConfig>(&json_str) {
            return Some(config);
        }
    }

    // Try simple format: viewport="375x667" or viewport="375x667@2"
    // Use byte-level parsing for speed
    let x_pos = memchr(b'x', bytes)?;

    let width_str = std::str::from_utf8(&bytes[..x_pos]).ok()?;
    let width: u32 = width_str.parse().ok()?;

    let rest = &bytes[x_pos + 1..];

    // Check for scale factor (@)
    if let Some(at_pos) = memchr(b'@', rest) {
        let height_str = std::str::from_utf8(&rest[..at_pos]).ok()?;
        let height: u32 = height_str.parse().ok()?;

        let scale_str = std::str::from_utf8(&rest[at_pos + 1..]).ok()?;
        let scale: f32 = scale_str.parse().ok()?;

        Some(ViewportConfig {
            width,
            height,
            device_scale_factor: Some(scale),
        })
    } else {
        let height_str = std::str::from_utf8(rest).ok()?;
        let height: u32 = height_str.parse().ok()?;

        Some(ViewportConfig {
            width,
            height,
            device_scale_factor: None,
        })
    }
}

/// Fast line counting using memchr iterator.
#[inline]
fn count_lines_fast(bytes: &[u8], pos: usize) -> u32 {
    let end = pos.min(bytes.len());
    memchr::memchr_iter(b'\n', &bytes[..end]).count() as u32 + 1
}

#[cfg(test)]
mod tests {
    use super::{parse_variants, parse_viewport};
    use crate::types::ArtParseError;
    use vize_carton::Bump;

    #[test]
    fn test_parse_single_variant() {
        let allocator = Bump::new();
        let content = r#"
  <variant name="Primary" default>
    <Button variant="primary">Click</Button>
  </variant>
"#;

        let result = parse_variants(&allocator, content, content, 0);
        assert!(result.is_ok());

        let variants = result.unwrap();
        assert_eq!(variants.len(), 1);
        assert_eq!(variants[0].name, "Primary");
        assert!(variants[0].is_default);
        insta::assert_debug_snapshot!(variants);
    }

    #[test]
    fn test_parse_multiple_variants() {
        let allocator = Bump::new();
        let content = r#"
  <variant name="Primary" default>
    <Button variant="primary">Primary</Button>
  </variant>
  <variant name="Secondary">
    <Button variant="secondary">Secondary</Button>
  </variant>
"#;

        let result = parse_variants(&allocator, content, content, 0);
        assert!(result.is_ok());

        let variants = result.unwrap();
        assert_eq!(variants.len(), 2);
        assert_eq!(variants[0].name, "Primary");
        assert_eq!(variants[1].name, "Secondary");
        assert!(variants[0].is_default);
        assert!(!variants[1].is_default);
    }

    #[test]
    fn test_parse_variant_with_args() {
        let allocator = Bump::new();
        let content = r#"
  <variant name="Custom" args='{"size":"lg","disabled":true}'>
    <Button>Custom</Button>
  </variant>
"#;

        let result = parse_variants(&allocator, content, content, 0);
        assert!(result.is_ok());

        let variants = result.unwrap();
        assert_eq!(variants[0].args.get("size"), Some(&serde_json::json!("lg")));
        assert_eq!(
            variants[0].args.get("disabled"),
            Some(&serde_json::json!(true))
        );
    }

    #[test]
    fn test_parse_viewport_simple() {
        let vp = parse_viewport(r#"viewport="375x667""#);
        assert!(vp.is_some());
        let vp = vp.unwrap();
        assert_eq!(vp.width, 375);
        assert_eq!(vp.height, 667);
    }

    #[test]
    fn test_parse_viewport_with_scale() {
        let vp = parse_viewport(r#"viewport="375x667@2""#);
        assert!(vp.is_some());
        let vp = vp.unwrap();
        assert_eq!(vp.width, 375);
        assert_eq!(vp.height, 667);
        assert_eq!(vp.device_scale_factor, Some(2.0));
    }

    #[test]
    fn test_parse_skip_vrt() {
        let allocator = Bump::new();
        let content = r#"<variant name="Test" skip-vrt><div></div></variant>"#;
        let result = parse_variants(&allocator, content, content, 0);
        assert!(result.is_ok());
        assert!(result.unwrap()[0].skip_vrt);
    }

    #[test]
    fn test_missing_name_error() {
        let allocator = Bump::new();
        let content = r#"<variant default><div></div></variant>"#;
        let result = parse_variants(&allocator, content, content, 0);
        assert!(matches!(
            result,
            Err(ArtParseError::MissingVariantName { .. })
        ));
    }
}
