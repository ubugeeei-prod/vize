//! Whole consumer packets with declared values, profiles and option custody.

use super::{Change, Parser, parse_with_compat, prove_named_after};
use vize_l0::{Allocator, cstr};
use vize_relief::{
    ExpressionNode, PropNode, RootNode, SimpleExpressionNode, TemplateChildNode,
    options::ParserOptions,
};

fn change(name: &'static str, before: &'static str, after: &'static str) -> Change {
    Change {
        directive: false,
        name,
        before,
        after,
    }
}

pub(in crate::parser) fn assert_complete_change(
    id: &str,
    source: &str,
    document: bool,
    options: ParserOptions,
    changes: &[Change],
) {
    let allocator = Allocator::new();
    let parser = if document {
        Parser::document_with_options(&allocator, source, options.clone())
    } else {
        Parser::with_options(&allocator, source, options.clone())
    };
    let (root, errors) = parser.parse();
    prove_named_after(
        id,
        source,
        document,
        &options,
        &cstr!("{root:#?}"),
        &cstr!("{errors:#?}"),
        changes,
    );
}

#[test]
fn complete_attribute_nodes_keep_text_raw_spans_errors_and_live_v_pre_exact() {
    let cases: &[(&str, &[Change])] = &[
        (
            "<p title='&fjlig;'>&fjlig;</p>",
            &[change("title", "f", "fj")],
        ),
        (
            "<p title='&NotEqualTilde;'>&NotEqualTilde;</p>",
            &[change("title", "≂", "≂\u{338}")],
        ),
        (
            "<p title='&ThickSpace;'>&ThickSpace;</p>",
            &[change("title", "\u{205f}", "\u{205f}\u{200a}")],
        ),
        (
            "<p v-pre title='&fjlig;'>&fjlig; {{x}}<i :title='&acE;'>{{ literal }}</i></p>{{normal}}",
            &[
                change("title", "f", "fj"),
                change(":title", "∾", "∾\u{333}"),
            ],
        ),
        (
            "<section v-pre><p title='&fjlig;'>&fjlig;{{x}}</p></section>{{normal}}",
            &[change("title", "f", "fj")],
        ),
    ];
    for (index, (source, changes)) in cases.iter().enumerate() {
        for document in [false, true] {
            assert_complete_change(
                cstr!("node-{index}-{document}").as_str(),
                source,
                document,
                ParserOptions::default(),
                changes,
            );
        }
    }
    for document in [false, true] {
        assert_complete_change(
            cstr!("custom-delimiters-{document}").as_str(),
            "<p title='&fjlig;'>[[value]] {{literal}} &fjlig;</p>",
            document,
            ParserOptions {
                delimiters: ("[[".into(), "]]".into()),
                ..ParserOptions::default()
            },
            &[change("title", "f", "fj")],
        );
        assert_complete_change(
            cstr!("tag-comments-{document}").as_str(),
            "<X // comment\n :title='&acE;'/>",
            document,
            ParserOptions {
                experimental_in_tag_comments: true,
                ..ParserOptions::default()
            },
            &[Change {
                directive: true,
                name: "bind",
                before: "∾",
                after: "∾\u{333}",
            }],
        );
        #[cfg(feature = "legacy")]
        assert_complete_change(
            cstr!("raw-interpolation-{document}").as_str(),
            "<p title='&fjlig;'>{{{raw}}} {{escaped}} &fjlig;</p>",
            document,
            ParserOptions {
                dialect: vize_l0::config::VueVersion::V1,
                ..ParserOptions::default()
            },
            &[change("title", "f", "fj")],
        );
    }
}

#[test]
fn ordinary_numeric_unknown_decode_once_and_recovery_nodes_keep_whole_before() {
    for source in [
        "<p title='&amp;&#32;&#x1f642;'>&amp;&#x1f642;</p>",
        "<p title='&amp;amp;'>&amp;amp;</p>",
        "<p title='&fjligx;&notit=;&ampx;&fjlig'>&unknown;</p>",
        "<p title='&#xzz;&#0;&#x110000;&'>text</p>",
        "<p title='unfinished",
    ] {
        for document in [false, true] {
            let before_allocator = Allocator::new();
            let after_allocator = Allocator::new();
            let before_parser = if document {
                Parser::new_document(&before_allocator, source)
            } else {
                Parser::new(&before_allocator, source)
            };
            let after_parser = if document {
                Parser::new_document(&after_allocator, source)
            } else {
                Parser::new(&after_allocator, source)
            };
            let before = parse_with_compat(before_parser);
            let after = after_parser.parse();
            assert_eq!(cstr!("{after:#?}"), cstr!("{before:#?}"), "{source}");
        }
    }
}

fn slot_expression<'r, 'a>(root: &'r RootNode<'a>) -> &'r SimpleExpressionNode<'a> {
    let TemplateChildNode::Element(element) = &root.children[0] else {
        panic!("authored component");
    };
    let PropNode::Directive(directive) = &element.props[0] else {
        panic!("authored v-slot");
    };
    assert_eq!(directive.name, "slot");
    let Some(ExpressionNode::Simple(expression)) = directive.exp.as_ref() else {
        panic!("complete direct slot value; refuse Compound or absent value");
    };
    expression
}

#[test]
fn retained_slot_role_receives_full_decoded_raw_before_consumers() {
    // Expected decoded bytes are authored here, independently of either route.
    // Full retained parameter/error packets must match the literal control.
    for (authored, literal, valid) in [
        ("{ '&fjlig;': item }", "{ 'fj': item }", true),
        ("{ '&NotEqualTilde;': item }", "{ '≂\u{338}': item }", true),
        ("{ '&amp;amp;': item }", "{ '&amp;': item }", true),
        ("{ '&fjlig;': }", "{ 'fj': }", false),
    ] {
        for document in [false, true] {
            let source = cstr!("<Panel v-slot=\"{authored}\"></Panel>");
            // Escape only the HTML carrier; the independently authored decoded
            // bytes above remain the complete expected JS producer input.
            let expected_attribute = literal.replace('&', "&#38;");
            let expected_source = cstr!("<Panel v-slot=\"{expected_attribute}\"></Panel>");
            let allocator = Allocator::new();
            let expected_allocator = Allocator::new();
            let parser = if document {
                Parser::new_document(&allocator, &source)
            } else {
                Parser::new(&allocator, &source)
            };
            let expected_parser = if document {
                Parser::new_document(&expected_allocator, &expected_source)
            } else {
                Parser::new(&expected_allocator, &expected_source)
            };
            let (root, errors) = parser.parse();
            let (expected_root, expected_errors) = expected_parser.parse();
            let expression = slot_expression(&root);
            let expected = slot_expression(&expected_root);
            assert_eq!(expression.content, literal);
            assert_eq!(
                source.get(expression.loc.span.start as usize..expression.loc.span.end as usize),
                Some(authored)
            );
            let retained = expression.js_ast.expect("original single producer role");
            let expected_retained = expected.js_ast.expect("literal control role");
            assert_eq!(retained.raw, literal);
            assert_eq!(
                cstr!("{retained:?}"),
                cstr!("{expected_retained:?}"),
                "whole retained role: {authored}"
            );
            assert_eq!(cstr!("{errors:#?}"), cstr!("{expected_errors:#?}"));
            assert_eq!(
                retained
                    .as_slot_parameters()
                    .expect("parameter role")
                    .is_ok(),
                valid
            );
        }
    }
}

#[test]
fn historical_ast_refuses_inheriting_the_private_full_parser_callback() {
    for document in [false, true] {
        // The real ParserCallbacks now opts in to full values. The explicit
        // historical owner must still supply f, with the whole AFTER fj.
        assert_complete_change(
            cstr!("before-owner-{document}").as_str(),
            "<p title='&fjlig;'>&fjlig;</p>",
            document,
            ParserOptions::default(),
            &[change("title", "f", "fj")],
        );
    }
}
