#![expect(clippy::disallowed_macros, reason = "fixtures use format!")]

use vize_armature::{Allocator, ParserOptions, TemplateChildNode, parse_with_options};

#[test]
fn lint_pragmas_follow_the_comment_option() {
    for pragma in [
        "eslint-disable",
        "eslint-disable-line",
        "eslint-disable-next-line",
        "eslint-enable",
        "oxlint-disable",
        "oxlint-disable-line",
        "oxlint-disable-next-line",
        "oxlint-enable",
    ] {
        let source = format!("<main><!-- {pragma} vue/no-v-html --><p>body</p></main>");
        for comments in [false, true] {
            let allocator = Allocator::new();
            let (root, errors) = parse_with_options(
                &allocator,
                &source,
                ParserOptions {
                    comments,
                    ..ParserOptions::default()
                },
            );
            assert!(errors.is_empty(), "{pragma}: {errors:?}");
            let TemplateChildNode::Element(main) = &root.children[0] else {
                panic!("expected main");
            };
            assert_eq!(
                main.children.len(),
                if comments { 2 } else { 1 },
                "{pragma}"
            );
            if comments {
                let TemplateChildNode::Comment(comment) = &main.children[0] else {
                    panic!("expected retained lint pragma");
                };
                assert!(comment.directive.is_some(), "{pragma}");
            }
            let TemplateChildNode::Element(body) = main.children.last().expect("body") else {
                panic!("expected body element");
            };
            assert_eq!(body.tag, "p");
        }
    }
}
