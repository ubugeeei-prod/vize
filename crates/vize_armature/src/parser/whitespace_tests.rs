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

/// Child list of `root.children[0]` as (kind, text) pairs; interpolations are
/// reported by their expression source.
fn vue2_preserve_children(source: &str) -> std::vec::Vec<(&'static str, std::string::String)> {
    vue2_preserve_children_with_comments(source, true)
}

fn vue2_preserve_children_with_comments(
    source: &str,
    comments: bool,
) -> std::vec::Vec<(&'static str, std::string::String)> {
    let allocator = Allocator::new();
    let (root, errors) = crate::parser::with_vue2_preserve_whitespace(|| {
        crate::parse_with_options(
            &allocator,
            source,
            ParserOptions {
                comments,
                is_pre_tag: |tag| tag == "pre",
                ..ParserOptions::default()
            },
        )
    });
    assert!(errors.is_empty(), "{errors:?}");
    let Some(TemplateChildNode::Element(element)) = root.children.first() else {
        panic!("expected a root element");
    };
    element
        .children
        .iter()
        .map(|child| match child {
            TemplateChildNode::Text(text) => ("text", text.content.to_string()),
            TemplateChildNode::Interpolation(interpolation) => (
                "interp",
                interpolation.loc.span.slice(source).trim().to_string(),
            ),
            TemplateChildNode::Element(element) => ("element", element.tag.to_string()),
            _ => ("other", std::string::String::new()),
        })
        .collect()
}

/// #8259. Expected values come from `@vue/compiler-sfc@2.7.16`
/// `compileTemplate({ compilerOptions: { whitespace: "preserve" } })`, which
/// merges each text/interpolation run into one `_v(...)` text node.
#[test]
fn vue2_preserve_keeps_whitespace_next_to_interpolations() {
    let source = "<p>\n    <span>\n      {{ n }}件\n    </span>\n    <b>{{ a }}</b>\n    {{ b }}\n    <i>x</i>\n  </p>";
    let p = vue2_preserve_children(source);
    let kinds: std::vec::Vec<_> = p
        .iter()
        .map(|(kind, text)| (*kind, text.as_str()))
        .collect();
    assert_eq!(
        kinds,
        [
            ("element", "span"),
            ("text", " "),
            ("element", "b"),
            ("text", "\n    "),
            ("interp", "{{ b }}"),
            ("text", "\n    "),
            ("element", "i"),
        ],
        "whitespace between elements condenses; whitespace beside {{ b }} stays as written; edges drop"
    );

    let span = vue2_preserve_children("<span>\n      {{ n }}件\n    </span>");
    let kinds: std::vec::Vec<_> = span
        .iter()
        .map(|(kind, text)| (*kind, text.as_str()))
        .collect();
    assert_eq!(
        kinds,
        [
            ("text", "\n      "),
            ("interp", "{{ n }}"),
            ("text", "件\n    "),
        ],
        "a text run with an interpolation keeps its leading and trailing whitespace"
    );
}

#[test]
fn vue2_preserve_whitespace_only_runs_and_pre() {
    let only_elements = vue2_preserve_children("<div>\n  <a></a>\n\n  <b></b>\n</div>");
    let kinds: std::vec::Vec<_> = only_elements
        .iter()
        .map(|(kind, text)| (*kind, text.as_str()))
        .collect();
    assert_eq!(kinds, [("element", "a"), ("text", " "), ("element", "b")]);

    let edge = vue2_preserve_children("<div>\n  hello {{ x }}\n</div>");
    let kinds: std::vec::Vec<_> = edge
        .iter()
        .map(|(kind, text)| (*kind, text.as_str()))
        .collect();
    assert_eq!(
        kinds,
        [
            ("text", "\n  hello "),
            ("interp", "{{ x }}"),
            ("text", "\n")
        ]
    );

    let mixed = vue2_preserve_children("<div>  a   b  <i></i></div>");
    let kinds: std::vec::Vec<_> = mixed
        .iter()
        .map(|(kind, text)| (*kind, text.as_str()))
        .collect();
    assert_eq!(kinds, [("text", "  a   b  "), ("element", "i")]);

    let pre = vue2_preserve_children("<pre>\n  a\n  <b> x </b>\n</pre>");
    let kinds: std::vec::Vec<_> = pre
        .iter()
        .map(|(kind, text)| (*kind, text.as_str()))
        .collect();
    assert_eq!(
        kinds,
        [("text", "  a\n  "), ("element", "b"), ("text", "\n")],
        "<pre> keeps its content apart from the ignored first newline"
    );
}

#[test]
fn vue2_preserve_keeps_removed_comments_as_text_run_boundaries() {
    let after_text = vue2_preserve_children_with_comments(
        "<p>\n  {{ a }}\n  <!-- explain -->\n  <b>x</b>\n</p>",
        false,
    );
    let kinds: std::vec::Vec<_> = after_text
        .iter()
        .map(|(kind, text)| (*kind, text.as_str()))
        .collect();
    assert_eq!(
        kinds,
        [
            ("text", "\n  "),
            ("interp", "{{ a }}"),
            ("text", "\n  "),
            ("text", " "),
            ("element", "b"),
        ]
    );

    let comment_first = vue2_preserve_children_with_comments(
        "<span>\n  <!-- keep on its own line -->\n  {{ kind }}</span>",
        false,
    );
    let kinds: std::vec::Vec<_> = comment_first
        .iter()
        .map(|(kind, text)| (*kind, text.as_str()))
        .collect();
    assert_eq!(kinds, [("text", "\n  "), ("interp", "{{ kind }}")]);
}

#[test]
fn vue2_preserve_drops_default_slot_tail_before_named_templates() {
    let children = vue2_preserve_children(
        "<MyCard>\n  <span>body</span>\n  <template #footer>f</template>\n</MyCard>",
    );
    let kinds: std::vec::Vec<_> = children
        .iter()
        .map(|(kind, text)| (*kind, text.as_str()))
        .collect();
    assert_eq!(kinds, [("element", "span"), ("element", "template")]);
}

#[test]
fn vue2_preserve_reports_the_preserve_strategy_and_restores_scope() {
    crate::parser::with_vue2_preserve_whitespace(|| {
        assert!(crate::parser::current_vue2_preserve());
        assert_eq!(
            crate::parser::current_whitespace_strategy(WhitespaceStrategy::Condense),
            WhitespaceStrategy::Preserve
        );
    });
    assert!(!crate::parser::current_vue2_preserve());
    crate::parser::with_whitespace_mode(WhitespaceStrategy::Preserve, false, || {
        assert!(!crate::parser::current_vue2_preserve());
    });
}

#[test]
fn comment_adjacent_condense_removes_spaces_but_preserve_keeps_them() {
    for source in [
        "<!--a--> <div></div>",
        "<div></div> <!--a-->",
        "<!--a--> <!--b-->",
        "<!--a-->\t<div></div>",
    ] {
        for (strategy, expected) in [
            (WhitespaceStrategy::Condense, 2),
            (WhitespaceStrategy::Preserve, 3),
        ] {
            let arena = Allocator::new();
            let (root, errors) = crate::parser::with_whitespace_strategy(strategy, || {
                crate::parse_with_options(&arena, source, ParserOptions::default())
            });
            assert!(errors.is_empty(), "{source}: {errors:?}");
            assert_eq!(root.children.len(), expected, "{source}: {strategy:?}");
        }
    }
}

#[test]
fn comment_adjacency_keeps_meaningful_unicode_text_and_pre_content() {
    for text in ["hello", "\u{00a0}", "\u{0085}", "\u{2003}"] {
        let arena = Allocator::new();
        let source = vize_l0::cstr!("<!--a-->{text}<div></div>");
        let (root, errors) = crate::parse_with_options(&arena, &source, ParserOptions::default());
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(root.children.len(), 3);
        let TemplateChildNode::Text(node) = &root.children[1] else {
            panic!("meaningful text must remain a real sibling");
        };
        assert_eq!(node.content, text);
    }
    let arena = Allocator::new();
    let (root, errors) = crate::parse_with_options(
        &arena,
        "<pre><!--a--> <i></i> <!--b--></pre>",
        ParserOptions {
            is_pre_tag: |tag| tag == "pre",
            ..ParserOptions::default()
        },
    );
    assert!(errors.is_empty(), "{errors:?}");
    let TemplateChildNode::Element(pre) = &root.children[0] else {
        panic!("expected pre");
    };
    assert_eq!(pre.children.len(), 5);
    for index in [1, 3] {
        let TemplateChildNode::Text(text) = &pre.children[index] else {
            panic!("pre whitespace must stay verbatim");
        };
        assert_eq!(text.content, " ");
    }
}
