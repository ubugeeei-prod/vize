//! The `<art>` custom block as an S1 surface tree.
//!
//! S0 hands over the block's whole element extent; S1 parses it into one
//! lossless tree whose root is the `<art>` element. Metadata reads the open
//! tag's attribute tokens, and variants are the tree's `<variant>` elements
//! (see [`super::variant`]) — no byte re-scan of the block.

use super::DefineArtMetadata;
use super::attrs::{attr_value, has_attr};
use super::status::{classify_status, is_unknown_status, unknown_status_warning};
use crate::types::{ArtMetadata, ArtParseError, ArtStatus};
use vize_s0::Allocator;
use vize_s1::{Element, OpenTag, SurfaceChild, SurfaceTree};

/// The `<art>` element at the root of the block's S1 tree.
///
/// The S0 frame starts at `<art`, so the first root child is the element in
/// every case S1 can build one; a frame S1 reads differently (the splitter's
/// tag-name alphabet is wider than HTML's, e.g. `<art:x>`) has no `<art>`.
pub(crate) fn art_element<'t, 'a>(tree: &'t SurfaceTree<'a>) -> Option<&'t Element<'a>> {
    match tree.children.first()? {
        SurfaceChild::Element(element) if element.tag() == "art" => Some(element),
        _ => None,
    }
}

/// Parse metadata from the `<art>` open tag, with `defineArt()` as the
/// fallback source for every field.
pub(crate) fn parse_metadata<'a>(
    allocator: &'a Allocator,
    open: &OpenTag<'a>,
    define_art: Option<&DefineArtMetadata<'a>>,
    filename: &str,
) -> Result<(ArtMetadata<'a>, vize_s0::Vec<'a, &'a str>), ArtParseError> {
    let title = attr_value(open, "title")
        .or_else(|| define_art.and_then(|metadata| metadata.title))
        .or_else(|| define_art.and_then(|metadata| metadata.component_name))
        .ok_or(ArtParseError::MissingTitle)?;

    // Optional attributes - all borrowed from source
    let description = attr_value(open, "description")
        .or_else(|| define_art.and_then(|metadata| metadata.description));
    let component = attr_value(open, "component")
        .or_else(|| define_art.and_then(|metadata| metadata.component));
    let category =
        attr_value(open, "category").or_else(|| define_art.and_then(|metadata| metadata.category));

    // Tags are comma-separated slices of the attribute value.
    let mut tags = vize_s0::Vec::new_in(&allocator);
    if let Some(tags_str) = attr_value(open, "tags") {
        for tag in tags_str.split(',') {
            let trimmed = tag.trim();
            if !trimmed.is_empty() {
                tags.push(trimmed);
            }
        }
    } else if let Some(define_art) = define_art {
        tags.extend(define_art.tags.iter().copied());
    }

    let attr_status = parse_status(open);
    let status = attr_status
        .or_else(|| define_art.and_then(|metadata| metadata.status.map(classify_status)))
        .unwrap_or_default();

    let mut warnings = vize_s0::Vec::new_in(&allocator);
    if let Some(value) = attr_value(open, "status").filter(|value| is_unknown_status(value)) {
        warnings.push(unknown_status_warning(allocator, filename, value));
    } else if attr_status.is_none()
        && let Some(value) = define_art
            .and_then(|metadata| metadata.status)
            .filter(|value| is_unknown_status(value))
    {
        warnings.push(unknown_status_warning(allocator, filename, value));
    }

    let order = attr_value(open, "order")
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

/// The `status` attribute, or the `draft` / `deprecated` shorthands.
fn parse_status(open: &OpenTag<'_>) -> Option<ArtStatus> {
    if let Some(status_str) = attr_value(open, "status") {
        Some(classify_status(status_str))
    } else if has_attr(open, "draft") {
        Some(ArtStatus::Draft)
    } else if has_attr(open, "deprecated") {
        Some(ArtStatus::Deprecated)
    } else {
        None
    }
}

#[cfg(test)]
#[expect(
    clippy::disallowed_macros,
    clippy::string_slice,
    reason = "tests assert by panicking, and insta snapshot macros expand through `std::format!`; see CONTRIBUTING.md, \"Snapshot assertions in test targets\""
)]
mod tests {
    use super::{art_element, parse_metadata, parse_status};
    use crate::types::ArtStatus;
    use vize_s0::Allocator;
    use vize_s1::parse;

    fn with_art<R>(source: &str, check: impl FnOnce(&Allocator, &vize_s1::Element<'_>) -> R) -> R {
        let allocator = Allocator::new();
        let (tree, _) = parse(&allocator, source);
        let art = art_element(&tree).expect("an <art> root element");
        check(&allocator, art)
    }

    /// The block geometry the pre-S1 snapshot pinned — the open tag's
    /// attribute text, the contents between the tags and their offset —
    /// now read off the S1 tree's tokens.
    #[derive(Debug)]
    #[expect(dead_code, reason = "read through the Debug snapshot")]
    struct BlockInfo<'a> {
        attrs_str: &'a str,
        content: &'a str,
        content_start: usize,
    }

    #[test]
    fn test_find_art_block() {
        let allocator = Allocator::new();
        let source = r#"<art title="Test"><variant name="A"></variant></art>"#;
        let (tree, _) = parse(&allocator, source);
        let art = art_element(&tree).expect("art");
        let frame = vize_s0::SourceRoot::new(source).unwrap().whole_block();
        let at = |slice: &str| frame.offset_of(slice).unwrap() as usize;
        let vize_s1::ElementClose::Present(close) = &art.close else {
            panic!("closed <art>");
        };
        let name_end = at(art.open.lt_name.text) + art.open.lt_name.text.len();
        let content_start = at(art.open.gt.text) + 1;
        let block = BlockInfo {
            attrs_str: source[name_end..at(art.open.gt.text)].trim(),
            content: &source[content_start..at(close.lt_slash_name.text)],
            content_start,
        };
        insta::assert_debug_snapshot!(block);
    }

    #[test]
    fn art_element_is_the_root_art_tag_only() {
        let allocator = Allocator::new();
        let (tree, _) = parse(
            &allocator,
            r#"<art title="Test"><variant name="A"></variant></art>"#,
        );
        let art = art_element(&tree).expect("art");
        assert_eq!(art.tag(), "art");
        assert_eq!(art.children.len(), 1);
        let (tree, _) = parse(&allocator, "<article></article>");
        assert!(art_element(&tree).is_none());
    }

    #[test]
    fn test_parse_metadata_minimal() {
        with_art(r#"<art title="Button"></art>"#, |allocator, art| {
            let (metadata, warnings) = parse_metadata(allocator, &art.open, None, "").unwrap();
            assert_eq!(metadata.title, "Button");
            assert_eq!(metadata.description, None);
            assert_eq!(metadata.status, ArtStatus::Ready);
            assert_eq!(warnings.len(), 0);
        });
    }

    #[test]
    fn test_parse_metadata_full() {
        let source = r#"<art title="Button" description="A button" category="atoms" tags="ui,input" status="draft"></art>"#;
        with_art(source, |allocator, art| {
            let (metadata, warnings) = parse_metadata(allocator, &art.open, None, "").unwrap();
            assert_eq!(metadata.title, "Button");
            assert_eq!(metadata.description, Some("A button"));
            assert_eq!(metadata.category, Some("atoms"));
            assert_eq!(metadata.tags.as_slice(), ["ui", "input"]);
            assert_eq!(metadata.status, ArtStatus::Draft);
            assert_eq!(warnings.len(), 0);
        });
    }

    #[test]
    fn test_parse_status() {
        let cases = [
            (r#"<art status="draft">"#, Some(ArtStatus::Draft)),
            (r#"<art status="ready">"#, Some(ArtStatus::Ready)),
            (r#"<art status="deprecated">"#, Some(ArtStatus::Deprecated)),
            (r#"<art status="wip">"#, Some(ArtStatus::Draft)),
            ("<art draft>", Some(ArtStatus::Draft)),
            ("<art deprecated>", Some(ArtStatus::Deprecated)),
            ("<art>", None),
        ];
        for (source, expected) in cases {
            let status = with_art(source, |_, art| parse_status(&art.open));
            assert_eq!(status, expected, "{source}");
        }
    }

    #[test]
    fn test_parse_metadata_unknown_status_warns() {
        with_art(
            r#"<art title="Button" status="wip"></art>"#,
            |allocator, art| {
                let (metadata, warnings) =
                    parse_metadata(allocator, &art.open, None, "button.art.vue").unwrap();
                assert_eq!(metadata.status, ArtStatus::Draft);
                assert_eq!(
                    warnings.as_slice(),
                    [
                        "button.art.vue: unknown status \"wip\"; falling back to \"draft\" (expected \"draft\" | \"ready\" | \"deprecated\")"
                    ]
                );
            },
        );
    }
}
