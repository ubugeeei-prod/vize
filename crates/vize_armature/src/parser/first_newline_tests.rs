//! A preserved comment is a first child; a dropped comment is not.
use super::{ParserOptions, parse_with_options};
use vize_l0::Allocator;
use vize_relief::{TemplateChildNode, options::WhitespaceStrategy};

#[test]
fn first_visible_child_respects_comments_in_both_whitespace_modes() {
    let source = "<pre><!-- keep -->\nline</pre>";
    for whitespace in [WhitespaceStrategy::Condense, WhitespaceStrategy::Preserve] {
        for comments in [false, true] {
            let allocator = Allocator::new();
            let (root, errors) = parse_with_options(
                &allocator,
                source,
                ParserOptions {
                    comments,
                    whitespace,
                    is_pre_tag: |tag| tag == "pre",
                    ..ParserOptions::default()
                },
            );
            assert!(errors.is_empty());
            let Some(TemplateChildNode::Element(pre)) = root.children.first() else {
                panic!("expected pre");
            };
            let Some(TemplateChildNode::Text(text)) = pre.children.last() else {
                panic!("expected text");
            };
            assert_eq!(text.content, if comments { "\nline" } else { "line" });
            assert_eq!(
                source.get(text.loc.span.start as usize..text.loc.span.end as usize),
                Some("\nline")
            );
            assert_eq!(
                matches!(pre.children.first(), Some(TemplateChildNode::Comment(_))),
                comments
            );
        }
    }
}
