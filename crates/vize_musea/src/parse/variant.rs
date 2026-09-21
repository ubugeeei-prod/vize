//! `<variant>` elements of the `<art>` S1 tree.
//!
//! A variant is an S1 element named `variant` anywhere under `<art>` —
//! never inside another variant, a comment or raw text, because those are
//! not elements in the tree. Its template is the authored bytes between
//! its open tag's `>` and its end tag, and every offset comes from the S0
//! frame the tree was parsed in.

mod values;

use super::attrs::{attr_value, has_attr};
use super::{calculate_location_fast, line_of};
use crate::types::{ArtParseError, ArtVariant};
use values::{parse_args_json, parse_viewport};
use vize_s0::{Allocator, SourceBlock, ToCompactString};
use vize_s1::{Element, ElementClose, SurfaceChild};

/// Collect every `<variant>` element under `children` (the `<art>`
/// element's contents), in document order.
///
/// The vector is a heap one: [`ArtVariant::args`] owns `serde_json` values, so
/// it cannot live in an arena container (Davinci P1-10). The walk keeps its
/// own stack, so nesting depth is bounded by memory, not the call stack.
pub(crate) fn parse_variants<'a>(
    allocator: &'a Allocator,
    children: &[SurfaceChild<'a>],
    frame: SourceBlock<'a>,
) -> Result<std::vec::Vec<ArtVariant<'a>>, ArtParseError> {
    let mut variants = std::vec::Vec::new();
    let mut levels = std::vec::Vec::from([children.iter()]);
    while let Some(level) = levels.last_mut() {
        let Some(child) = level.next() else {
            levels.pop();
            continue;
        };
        let SurfaceChild::Element(element) = child else {
            continue;
        };
        if element.tag() == "variant" {
            variants.push(parse_single_variant(allocator, element, frame)?);
        } else {
            levels.push(element.children.iter());
        }
    }
    Ok(variants)
}

/// Read one `<variant>` element. All strings are slices of the source —
/// zero allocations except for the decoded `args` JSON.
fn parse_single_variant<'a>(
    allocator: &'a Allocator,
    element: &Element<'a>,
    frame: SourceBlock<'a>,
) -> Result<ArtVariant<'a>, ArtParseError> {
    let source = frame.root_source();
    let open = &element.open;
    let start = offset_in(frame, open.lt_name.text);
    let line = line_of(source, start);
    if open.gt.is_missing() {
        return Err(ArtParseError::ParseError {
            line,
            message: "Unclosed <variant> tag".to_compact_string(),
        });
    }

    let name = attr_value(open, "name").ok_or(ArtParseError::MissingVariantName { line })?;
    let is_default = has_attr(open, "default");
    let skip_vrt = has_attr(open, "skip-vrt") || has_attr(open, "skipVrt");
    let args = attr_value(open, "args")
        .and_then(|s| parse_args_json(allocator, s).ok())
        .unwrap_or_default();
    let viewport = attr_value(open, "viewport").and_then(parse_viewport);

    let body_start = end_in(frame, open.gt.text);
    let (body_end, end) = match &element.close {
        ElementClose::Present(close) => (
            offset_in(frame, close.lt_slash_name.text),
            end_in(frame, close.gt.text),
        ),
        // `<variant … />`: an authored empty body.
        ElementClose::NotExpected => (body_start, body_start),
        ElementClose::Missing | ElementClose::Implicit => {
            return Err(ArtParseError::ParseError {
                line,
                message: "Missing </variant> closing tag".to_compact_string(),
            });
        }
    };
    let template = source
        .get(body_start as usize..body_end as usize)
        .unwrap_or_default()
        .trim();

    Ok(ArtVariant {
        name,
        template,
        is_default,
        args,
        viewport,
        skip_vrt,
        loc: Some(calculate_location_fast(source, start, end)),
    })
}

/// File-absolute offset of an S1 token slice.
fn offset_in(frame: SourceBlock<'_>, slice: &str) -> u32 {
    let offset = frame.offset_of(slice);
    debug_assert!(offset.is_some(), "S1 tokens are slices of the parsed frame");
    offset.unwrap_or_else(|| frame.start())
}

/// File-absolute offset just past an S1 token slice.
fn end_in(frame: SourceBlock<'_>, slice: &str) -> u32 {
    offset_in(frame, slice) + slice.len() as u32
}

#[cfg(test)]
mod tests {
    use super::parse_variants;
    use crate::types::{ArtParseError, ArtVariant};
    use vize_s0::{Allocator, SourceRoot};
    use vize_s1::parse;

    /// Parse `content` as the `<art>` block's contents, framed at offset 0.
    fn variants_of<'a>(
        allocator: &'a Allocator,
        content: &'a str,
    ) -> Result<std::vec::Vec<ArtVariant<'a>>, ArtParseError> {
        let (tree, _) = parse(allocator, content);
        let frame = SourceRoot::new(content)
            .expect("small source")
            .whole_block();
        parse_variants(allocator, &tree.children, frame)
    }

    fn shape<'a>(variants: &[ArtVariant<'a>]) -> std::vec::Vec<(&'a str, &'a str)> {
        variants.iter().map(|v| (v.name, v.template)).collect()
    }

    #[test]
    fn test_parse_single_variant() {
        let allocator = Allocator::new();
        let content = r#"
  <variant name="Primary" default>
    <Button variant="primary">Click</Button>
  </variant>
"#;

        let variants = variants_of(&allocator, content).unwrap();
        assert_eq!(variants.len(), 1);
        assert_eq!(variants[0].name, "Primary");
        assert!(variants[0].is_default);
        insta::assert_debug_snapshot!(variants);
    }

    #[test]
    fn test_parse_multiple_variants() {
        let allocator = Allocator::new();
        let content = r#"
  <variant name="Primary" default>
    <Button variant="primary">Primary</Button>
  </variant>
  <variant name="Secondary">
    <Button variant="secondary">Secondary</Button>
  </variant>
"#;

        let variants = variants_of(&allocator, content).unwrap();
        let names: std::vec::Vec<_> = variants.iter().map(|v| (v.name, v.is_default)).collect();
        assert_eq!(names, [("Primary", true), ("Secondary", false)]);
    }

    #[test]
    fn test_parse_variant_with_args() {
        let allocator = Allocator::new();
        let content = r#"
  <variant name="Custom" args='{"size":"lg","disabled":true}'>
    <Button>Custom</Button>
  </variant>
"#;

        let variants = variants_of(&allocator, content).unwrap();
        assert_eq!(variants[0].args.len(), 2);
        assert_eq!(variants[0].args.get("size"), Some(&serde_json::json!("lg")));
        assert_eq!(
            variants[0].args.get("disabled"),
            Some(&serde_json::json!(true))
        );
    }

    #[test]
    fn test_parse_variant_args_decode_entities() {
        let allocator = Allocator::new();
        let content = r#"<variant name="E" args="{&quot;label&quot;:&quot;a &lt;b&gt; &amp; c&quot;}"></variant>"#;
        let variants = variants_of(&allocator, content).unwrap();
        assert_eq!(variants[0].args.len(), 1);
        assert_eq!(
            variants[0].args.get("label"),
            Some(&serde_json::json!("a <b> & c"))
        );
    }

    #[test]
    fn test_parse_skip_vrt() {
        let allocator = Allocator::new();
        let content = r#"<variant name="A" skip-vrt><div></div></variant><variant name="B" skipVrt></variant><variant name="C"></variant>"#;
        let variants = variants_of(&allocator, content).unwrap();
        let flags: std::vec::Vec<_> = variants.iter().map(|v| (v.name, v.skip_vrt)).collect();
        assert_eq!(flags, [("A", true), ("B", true), ("C", false)]);
    }

    #[test]
    fn test_missing_name_error() {
        let allocator = Allocator::new();
        let content = "\n<variant default><div></div></variant>";
        assert!(matches!(
            variants_of(&allocator, content),
            Err(ArtParseError::MissingVariantName { line: 2 })
        ));
    }

    #[test]
    fn unclosed_open_tag_is_a_parse_error() {
        let allocator = Allocator::new();
        let Err(ArtParseError::ParseError { line, message }) =
            variants_of(&allocator, "<variant name=\"A\"")
        else {
            panic!("expected a parse error");
        };
        assert_eq!((line, message.as_str()), (1, "Unclosed <variant> tag"));
    }

    #[test]
    fn missing_end_tag_is_a_parse_error() {
        let allocator = Allocator::new();
        let Err(ArtParseError::ParseError { line, message }) =
            variants_of(&allocator, "\n\n<variant name=\"A\"><div>x</div>")
        else {
            panic!("expected a parse error");
        };
        assert_eq!(
            (line, message.as_str()),
            (3, "Missing </variant> closing tag")
        );
    }

    #[test]
    fn variants_are_found_through_wrappers_but_not_inside_variants() {
        let allocator = Allocator::new();
        let content = r#"<div><variant name="Wrapped">w</variant></div><variant name="Outer"><variant name="Inner">i</variant></variant>"#;
        let variants = variants_of(&allocator, content).unwrap();
        assert_eq!(
            shape(&variants),
            [
                ("Wrapped", "w"),
                ("Outer", r#"<variant name="Inner">i</variant>"#)
            ]
        );
    }

    #[test]
    fn self_closing_variant_has_an_empty_body() {
        let allocator = Allocator::new();
        let content = r#"<variant name="Empty" /><variant name="Next">n</variant>"#;
        let variants = variants_of(&allocator, content).unwrap();
        let locs: std::vec::Vec<_> = variants
            .iter()
            .map(|v| v.loc.map(|loc| (loc.start, loc.end)))
            .collect();
        assert_eq!(shape(&variants), [("Empty", ""), ("Next", "n")]);
        assert_eq!(locs, [Some((0, 24)), Some((24, 56))]);
    }
}
