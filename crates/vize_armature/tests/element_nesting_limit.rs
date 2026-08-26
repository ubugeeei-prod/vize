//! Behaviour of the element-nesting depth guard.
//!
//! The parser stops descending after `MAX_ELEMENT_NESTING_DEPTH` open elements so
//! the AST it hands to the recursive later passes stays bounded. That guard used
//! to leak into the diagnostics: over-limit elements were attached as leaves
//! without being pushed onto the open-element stack, so their perfectly valid end
//! tags found nothing to close and were reported as `InvalidEndTag`. The error
//! count then grew as `2 * (depth - limit)`, and every one of those `InvalidEndTag`
//! diagnostics pointed at correct user code.
//!
//! Test-only: `std::string::String` and `format!` build the deeply nested source
//! strings that the parser is fed here.
#![allow(clippy::disallowed_macros, clippy::disallowed_types)]

use vize_armature::parse;
use vize_relief::{
    TemplateChildNode,
    errors::{CompilerError, ErrorCode},
};
use vize_s0::Allocator;

/// Mirrors `MAX_ELEMENT_NESTING_DEPTH` in `parser::element::nesting`.
const NESTING_LIMIT: usize = 4096;

const TOO_DEEP: &str = "Element nesting is too deep.";

fn nested_divs(depth: usize) -> String {
    let mut source = String::with_capacity(depth * 11 + 1);
    for _ in 0..depth {
        source.push_str("<div>");
    }
    source.push('x');
    for _ in 0..depth {
        source.push_str("</div>");
    }
    source
}

fn diagnostics(errors: &[CompilerError]) -> Vec<(ErrorCode, &str)> {
    errors
        .iter()
        .map(|error| (error.code, error.message.as_str()))
        .collect()
}

fn element_depth(node: Option<&TemplateChildNode<'_>>) -> usize {
    let mut node = node;
    let mut depth = 0;
    while let Some(TemplateChildNode::Element(element)) = node {
        depth += 1;
        node = element.children.first();
    }
    depth
}

#[test]
fn nesting_at_the_limit_parses_without_diagnostics() {
    let allocator = Allocator::new();
    let source = nested_divs(NESTING_LIMIT);
    let (root, errors) = parse(&allocator, &source);

    assert_eq!(diagnostics(&errors), Vec::new());
    assert_eq!(element_depth(root.children.first()), NESTING_LIMIT);
}

#[test]
fn nesting_past_the_limit_reports_the_limit_once_without_inventing_end_tag_errors() {
    for depth in [NESTING_LIMIT + 1, NESTING_LIMIT + 2, 5_000, 10_000] {
        let allocator = Allocator::new();
        let source = nested_divs(depth);
        let (_, errors) = parse(&allocator, &source);

        assert_eq!(
            diagnostics(&errors),
            vec![(ErrorCode::ExtendPoint, TOO_DEEP)],
            "depth {depth}"
        );
    }
}

#[test]
fn each_over_limit_region_is_reported_at_its_own_location() {
    // Two independent subtrees that each cross the limit. The recovery is
    // per-region, so each one is worth a diagnostic; neither should produce an
    // `InvalidEndTag`.
    let allocator = Allocator::new();
    let deep = nested_divs(NESTING_LIMIT + 1);
    let source = format!("<section>{deep}{deep}</section>");
    let (_, errors) = parse(&allocator, &source);

    assert_eq!(
        diagnostics(&errors),
        vec![
            (ErrorCode::ExtendPoint, TOO_DEEP),
            (ErrorCode::ExtendPoint, TOO_DEEP),
        ]
    );
}

#[test]
fn retained_tree_depth_stays_bounded_past_the_limit() {
    let allocator = Allocator::new();
    let source = nested_divs(5_000);
    let (root, errors) = parse(&allocator, &source);

    assert_eq!(
        diagnostics(&errors),
        vec![(ErrorCode::ExtendPoint, TOO_DEEP)]
    );
    // The over-limit elements are kept, but flattened onto the deepest retained
    // parent rather than nested, so the tree the later passes recurse into is
    // one level deeper than the limit and no more.
    assert_eq!(element_depth(root.children.first()), NESTING_LIMIT + 1);
}

#[test]
fn genuinely_unmatched_end_tags_are_still_reported_past_the_limit() {
    // The suppression must be limited to end tags that really do close an
    // over-limit element. A stray `</span>` inside the over-limit region has
    // nothing to close in the source either, and must keep its diagnostic.
    let allocator = Allocator::new();
    let mut source = String::new();
    for _ in 0..NESTING_LIMIT + 1 {
        source.push_str("<div>");
    }
    source.push_str("</span>");
    for _ in 0..NESTING_LIMIT + 1 {
        source.push_str("</div>");
    }

    let (_, errors) = parse(&allocator, &source);

    assert_eq!(
        diagnostics(&errors),
        vec![
            (ErrorCode::ExtendPoint, TOO_DEEP),
            (ErrorCode::InvalidEndTag, "Invalid end tag."),
        ]
    );
}
