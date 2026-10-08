use super::is_whitespace_text;
use crate::{
    Allocator, ParserOptions, SourceLocation, TemplateChildNode, TextNode, WhitespaceStrategy,
};

#[test]
fn whitespace_only_text_uses_the_complete_vue_ascii_alphabet() {
    for (content, expected) in [
        ("", true),
        (" ", true),
        ("\t", true),
        ("\n", true),
        ("\u{000C}", true),
        ("\r", true),
        (" \t\r\n\u{000C} ", true),
        ("\u{000B}", false),
        ("\u{0085}", false),
        ("\u{00A0}", false),
        ("\u{2003}", false),
        ("\u{2028}", false),
        ("\u{2029}", false),
        ("\u{3000}", false),
        ("é", false),
        ("字", false),
        ("🙂", false),
        (" \t\u{00A0}\n ", false),
    ] {
        let allocator = Allocator::new();
        let text = TemplateChildNode::Text(vize_l0::Box::new_in(
            TextNode::new(content, SourceLocation::default()),
            &&allocator,
        ));
        assert_eq!(is_whitespace_text(&text), expected, "{content:?}");
    }
}

#[test]
fn unicode_text_and_spans_survive_all_whitespace_modes() {
    let source = "<p>\u{00A0}é\u{2003}字\u{2028}🙂\u{3000}</p>";
    let expected = "\u{00A0}é\u{2003}字\u{2028}🙂\u{3000}";
    for whitespace in [WhitespaceStrategy::Condense, WhitespaceStrategy::Preserve] {
        for legacy in [false, true] {
            let allocator = Allocator::new();
            let (root, errors) = crate::parser::with_whitespace_mode(whitespace, legacy, || {
                crate::parse_with_options(&allocator, source, ParserOptions::default())
            });
            assert!(errors.is_empty(), "{whitespace:?}/{legacy}: {errors:?}");
            assert_eq!(root.children.len(), 1);
            let TemplateChildNode::Element(element) = &root.children[0] else {
                panic!("expected p");
            };
            assert_eq!(element.tag, "p");
            assert_eq!(element.children.len(), 1);
            let TemplateChildNode::Text(text) = &element.children[0] else {
                panic!("expected Unicode text");
            };
            assert_eq!(text.content, expected);
            assert_eq!(text.loc.span.slice(source), expected);
            assert_eq!(text.loc.span.start as usize, 3);
            assert_eq!(text.loc.span.end as usize, source.len() - 4);
            assert_eq!(text.content.as_ptr(), source[3..].as_ptr());
        }
    }
}

#[test]
fn ascii_whitespace_keeps_between_element_spans_in_all_modes() {
    let source = "<p> \t\r\n\u{000C}<i></i>\u{000C}<b></b>\n\r\t </p>";
    for whitespace in [WhitespaceStrategy::Condense, WhitespaceStrategy::Preserve] {
        for legacy in [false, true] {
            let allocator = Allocator::new();
            let (root, errors) = crate::parser::with_whitespace_mode(whitespace, legacy, || {
                crate::parse_with_options(&allocator, source, ParserOptions::default())
            });
            assert!(errors.is_empty(), "{whitespace:?}/{legacy}: {errors:?}");
            assert_eq!(root.children.len(), 1);
            let TemplateChildNode::Element(element) = &root.children[0] else {
                panic!("expected p");
            };
            assert_eq!(element.tag, "p");
            assert_eq!(element.children.len(), 3);
            let TemplateChildNode::Element(first) = &element.children[0] else {
                panic!("expected i");
            };
            assert_eq!(first.tag, "i");
            let TemplateChildNode::Text(text) = &element.children[1] else {
                panic!("expected condensed whitespace");
            };
            assert_eq!(text.content, " ");
            assert_eq!(text.loc.span.slice(source), "\u{000C}");
            let TemplateChildNode::Element(last) = &element.children[2] else {
                panic!("expected b");
            };
            assert_eq!(last.tag, "b");
        }
    }
}
