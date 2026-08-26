#![allow(clippy::disallowed_macros)]

use super::{parse, parse_with_options};
use vize_relief::{
    CommentKind, PropNode, TemplateChildNode, errors::ErrorCode, options::ParserOptions,
};
use vize_s0::Allocator;

#[test]
fn test_parse_experimental_in_tag_comments() {
    let allocator = Allocator::new();
    let (root, errors) = parse_with_options(
        &allocator,
        "<LegacySelect\n  :options=\"options\"\n  // @vue-expect-error legacy API\n  :selected-id=\"selectedId\"\n/>",
        ParserOptions {
            experimental_in_tag_comments: true,
            ..ParserOptions::default()
        },
    );

    assert!(errors.is_empty());
    assert_eq!(root.comments.len(), 1);
    assert_eq!(root.comments[0].kind, CommentKind::InTag);
    assert_eq!(root.comments[0].content, " @vue-expect-error legacy API");

    let TemplateChildNode::Element(el) = &root.children[0] else {
        panic!("Expected element");
    };
    assert_eq!(el.props.len(), 2);
    assert!(
        root.children
            .iter()
            .all(|child| !matches!(child, TemplateChildNode::Comment(_)))
    );
}

#[test]
fn test_parse_in_tag_comments_are_opt_in() {
    let allocator = Allocator::new();
    let (_root, errors) = parse(&allocator, "<LegacySelect\n  // note\n/>");

    assert!(
        errors
            .iter()
            .any(|error| error.code == ErrorCode::UnexpectedSolidusInTag)
    );
}

#[test]
fn test_parse_in_tag_comments_preserve_when_comments_disabled() {
    let allocator = Allocator::new();
    let (root, errors) = parse_with_options(
        &allocator,
        "<LegacySelect\n  // note\n/>",
        ParserOptions {
            comments: false,
            experimental_in_tag_comments: true,
            ..ParserOptions::default()
        },
    );

    assert!(errors.is_empty());
    assert_eq!(root.comments.len(), 1);
    assert_eq!(root.children.len(), 1);
}

#[test]
fn test_parse_slash_slash_inside_attribute_value_is_not_in_tag_comment() {
    let allocator = Allocator::new();
    let (root, errors) = parse_with_options(
        &allocator,
        r#"<div title="not // a comment"></div>"#,
        ParserOptions {
            experimental_in_tag_comments: true,
            ..ParserOptions::default()
        },
    );

    assert!(errors.is_empty());
    assert!(root.comments.is_empty());
    let TemplateChildNode::Element(el) = &root.children[0] else {
        panic!("Expected element");
    };
    let PropNode::Attribute(attr) = &el.props[0] else {
        panic!("Expected attribute");
    };
    assert_eq!(
        attr.value.as_ref().map(|value| value.content),
        Some("not // a comment")
    );
}
