//! Parser for the `<art>` block.
//!
//! High-performance parser for extracting the `<art>` block and its metadata.

use super::DefineArtMetadata;
use super::status::{classify_status, is_unknown_status, unknown_status_warning};
use super::{BlockInfo, extract_attr, has_attr};
use crate::types::{ArtMetadata, ArtParseError, ArtStatus};
use memchr::{memchr, memmem};
use vize_carton::Allocator;

/// Find the `<art>` block in the source.
/// Returns the block info with attributes and content.
#[inline]
pub(crate) fn find_art_block<'a>(
    bytes: &[u8],
    source: &'a str,
) -> Result<BlockInfo<'a>, ArtParseError> {
    // Use memmem for fast substring search
    let art_finder = memmem::Finder::new(b"<art");

    let Some(art_start) = art_finder.find(bytes) else {
        return Err(ArtParseError::NoArtBlock);
    };

    // Verify it's actually <art and not <article etc
    let after_art = art_start + 4;
    if after_art < bytes.len() {
        let next_char = bytes[after_art];
        if next_char != b' ' && next_char != b'>' && next_char != b'\n' && next_char != b'\t' {
            // Not <art, keep searching
            // For simplicity, return NoArtBlock - could recurse for robustness
            return Err(ArtParseError::NoArtBlock);
        }
    }

    // Find '>' that closes the opening tag
    let Some(tag_close_offset) = memchr(b'>', &bytes[art_start..]) else {
        return Err(ArtParseError::NoArtBlock);
    };
    let tag_end = art_start + tag_close_offset;

    // Extract attributes (skip "<art")
    let attrs_start = art_start + 4;
    let attrs_str = source[attrs_start..tag_end].trim();

    // Find </art>
    let content_start = tag_end + 1;
    let close_finder = memmem::Finder::new(b"</art>");
    let Some(close_offset) = close_finder.find(&bytes[content_start..]) else {
        return Err(ArtParseError::NoArtBlock);
    };
    let close_pos = content_start + close_offset;

    let content = &source[content_start..close_pos];

    Ok(BlockInfo {
        attrs_str,
        content,
        content_start,
    })
}

/// Parse metadata from `<art>` block attributes.
/// Uses arena allocation for tags vector.
#[inline]
pub(crate) fn parse_metadata<'a>(
    allocator: &'a Allocator,
    block: &BlockInfo<'a>,
    define_art: Option<&DefineArtMetadata<'a>>,
    filename: &str,
) -> Result<(ArtMetadata<'a>, vize_carton::Vec<'a, &'a str>), ArtParseError> {
    let attrs = block.attrs_str;

    let title = extract_attr(attrs, "title")
        .or_else(|| define_art.and_then(|metadata| metadata.title))
        .or_else(|| define_art.and_then(|metadata| metadata.component_name))
        .ok_or(ArtParseError::MissingTitle)?;

    // Optional attributes - all borrowed from source
    let description = extract_attr(attrs, "description")
        .or_else(|| define_art.and_then(|metadata| metadata.description));
    let component = extract_attr(attrs, "component")
        .or_else(|| define_art.and_then(|metadata| metadata.component));
    let category = extract_attr(attrs, "category")
        .or_else(|| define_art.and_then(|metadata| metadata.category));

    // Parse tags (comma-separated) into arena-allocated vec
    let mut tags = vize_carton::Vec::new_in(&allocator);
    if let Some(tags_str) = extract_attr(attrs, "tags") {
        // Split by comma, trim each tag - no allocations, just slices
        for tag in tags_str.split(',') {
            let trimmed = tag.trim();
            if !trimmed.is_empty() {
                tags.push(trimmed);
            }
        }
    } else if let Some(define_art) = define_art {
        tags.extend(define_art.tags.iter().copied());
    }

    let attr_status = parse_status(attrs);
    let status = attr_status
        .or_else(|| define_art.and_then(|metadata| metadata.status.map(classify_status)))
        .unwrap_or_default();

    let mut warnings = vize_carton::Vec::new_in(&allocator);
    if let Some(value) = extract_attr(attrs, "status").filter(|value| is_unknown_status(value)) {
        warnings.push(unknown_status_warning(allocator, filename, value));
    } else if attr_status.is_none()
        && let Some(value) = define_art
            .and_then(|metadata| metadata.status)
            .filter(|value| is_unknown_status(value))
    {
        warnings.push(unknown_status_warning(allocator, filename, value));
    }

    // Parse order
    let order = extract_attr(attrs, "order")
        .and_then(|s| s.parse::<u32>().ok())
        .or_else(|| define_art.and_then(|metadata| metadata.order));

    Ok((
        ArtMetadata {
            title,
            description,
            component,
            category,
            tags,
            status,
            order,
        },
        warnings,
    ))
}

/// Parse the status attribute value.
#[inline]
fn parse_status(attrs: &str) -> Option<ArtStatus> {
    if let Some(status_str) = extract_attr(attrs, "status") {
        Some(classify_status(status_str))
    } else if has_attr(attrs, "draft") {
        Some(ArtStatus::Draft)
    } else if has_attr(attrs, "deprecated") {
        Some(ArtStatus::Deprecated)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{find_art_block, parse_metadata, parse_status};
    use crate::types::ArtStatus;
    use vize_carton::Allocator;

    #[test]
    fn test_find_art_block() {
        let source = r#"<art title="Test"><variant name="A"></variant></art>"#;
        let result = find_art_block(source.as_bytes(), source);
        assert!(result.is_ok());

        let block = result.unwrap();
        insta::assert_debug_snapshot!(block);
    }

    #[test]
    fn test_parse_metadata_minimal() {
        let allocator = Allocator::new();
        let source = r#"<art title="Button"></art>"#;
        let block = find_art_block(source.as_bytes(), source).unwrap();
        let (metadata, warnings) = parse_metadata(&allocator, &block, None, "").unwrap();

        assert_eq!(metadata.title, "Button");
        assert_eq!(metadata.description, None);
        assert_eq!(metadata.status, ArtStatus::Ready);
        assert_eq!(warnings.len(), 0);
    }

    #[test]
    fn test_parse_metadata_full() {
        let allocator = Allocator::new();
        let source = r#"<art title="Button" description="A button" category="atoms" tags="ui,input" status="draft"></art>"#;
        let block = find_art_block(source.as_bytes(), source).unwrap();
        let (metadata, warnings) = parse_metadata(&allocator, &block, None, "").unwrap();

        assert_eq!(metadata.title, "Button");
        assert_eq!(metadata.description, Some("A button"));
        assert_eq!(metadata.category, Some("atoms"));
        assert_eq!(metadata.tags.len(), 2);
        assert_eq!(metadata.tags[0], "ui");
        assert_eq!(metadata.tags[1], "input");
        assert_eq!(metadata.status, ArtStatus::Draft);
        assert_eq!(warnings.len(), 0);
    }

    #[test]
    fn test_parse_status() {
        assert_eq!(parse_status(r#"status="draft""#), Some(ArtStatus::Draft));
        assert_eq!(parse_status(r#"status="ready""#), Some(ArtStatus::Ready));
        assert_eq!(
            parse_status(r#"status="deprecated""#),
            Some(ArtStatus::Deprecated)
        );
        assert_eq!(parse_status(r#"status="wip""#), Some(ArtStatus::Draft));
        assert_eq!(parse_status(r#"draft"#), Some(ArtStatus::Draft));
        assert_eq!(parse_status(r#"deprecated"#), Some(ArtStatus::Deprecated));
        assert_eq!(parse_status(r#""#), None);
    }

    #[test]
    fn test_parse_metadata_unknown_status_warns() {
        let allocator = Allocator::new();
        let source = r#"<art title="Button" status="wip"></art>"#;
        let block = find_art_block(source.as_bytes(), source).unwrap();
        let (metadata, warnings) =
            parse_metadata(&allocator, &block, None, "button.art.vue").unwrap();
        assert_eq!(metadata.status, ArtStatus::Draft);
        assert_eq!(
            warnings.as_slice(),
            [
                "button.art.vue: unknown status \"wip\"; falling back to \"draft\" (expected \"draft\" | \"ready\" | \"deprecated\")"
            ]
        );
    }
}
