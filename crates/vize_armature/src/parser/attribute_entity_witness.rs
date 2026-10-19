//! Named BEFORE/AFTER consumer witnesses, never normalization of actual output.
//!
//! BEFORE executes the unchanged public first-scalar compatibility boundary.
//! An authored attribute-only mutation of that whole AST supplies expected
//! AFTER. Current production captures must equal every expected field, and
//! complete errors/raw spans stay exact. Original frozen bins remain history.

use super::{Parser, compat_reference::parse_with_compat};
use vize_l0::{Allocator, String, cstr};
use vize_relief::{ExpressionNode, PropNode, TemplateChildNode, options::ParserOptions};

#[derive(Clone, Copy)]
pub(super) struct Change {
    pub(super) directive: bool,
    pub(super) name: &'static str,
    pub(super) before: &'static str,
    pub(super) after: &'static str,
}

fn apply_authored_changes(
    children: &mut [TemplateChildNode<'_>],
    changes: &[Change],
    hits: &mut [usize],
) {
    for child in children {
        let TemplateChildNode::Element(element) = child else {
            continue;
        };
        for prop in &mut element.props {
            for (index, change) in changes.iter().enumerate() {
                let content = match prop {
                    PropNode::Attribute(attribute)
                        if !change.directive && attribute.name == change.name =>
                    {
                        attribute.value.as_mut().map(|value| &mut value.content)
                    }
                    PropNode::Directive(directive)
                        if change.directive && directive.name == change.name =>
                    {
                        // The two frozen option witnesses have invalid expression
                        // bytes before and after. No JS AST is rewritten here.
                        let Some(ExpressionNode::Simple(expression)) = directive.exp.as_mut()
                        else {
                            panic!(
                                "authored direct Simple expression; refuse Compound or absent value"
                            );
                        };
                        assert!(expression.js_ast.is_none(), "no retained-AST workaround");
                        Some(&mut expression.content)
                    }
                    _ => None,
                };
                if let Some(content) = content {
                    assert_eq!(*content, change.before, "whole BEFORE value");
                    *content = change.after;
                    hits[index] += 1;
                }
            }
        }
        apply_authored_changes(&mut element.children, changes, hits);
    }
}

pub(super) fn prove_named_after(
    id: &str,
    source: &str,
    document: bool,
    options: &ParserOptions,
    actual_tree: &str,
    actual_errors: &str,
    changes: &[Change],
) -> String {
    let allocator = Allocator::new();
    let parser = if document {
        Parser::document_with_options(&allocator, source, options.clone())
    } else {
        Parser::with_options(&allocator, source, options.clone())
    };
    let (mut before, errors) = parse_with_compat(parser);
    let historical_tree = cstr!("{before:#?}");
    let historical_errors = cstr!("{errors:#?}");
    let option_fields = cstr!("{options:?}");
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/differential/attribute-entity-consumer-8142");
    std::fs::create_dir_all(&directory).expect("whole named consumer packets");
    let mut raw = Vec::new();
    for field in [
        source,
        option_fields.as_str(),
        historical_tree.as_str(),
        historical_errors.as_str(),
        actual_tree,
        actual_errors,
    ] {
        raw.extend_from_slice(&(field.len() as u64).to_le_bytes());
        raw.extend_from_slice(field.as_bytes());
    }
    std::fs::write(directory.join(cstr!("{id}.raw.bin").as_str()), raw)
        .expect("retain actual whole packets before judging");
    let mut hits = vec![0; changes.len()];
    apply_authored_changes(&mut before.children, changes, &mut hits);
    let expected_tree = cstr!("{before:#?}");
    // Preserve whole original source/options, true BEFORE and actual AFTER
    // before any assertion. AFTER is never reduced to a historical projection.
    let fields = [
        source,
        option_fields.as_str(),
        historical_tree.as_str(),
        historical_errors.as_str(),
        expected_tree.as_str(),
        actual_tree,
        actual_errors,
    ];
    let mut packet = Vec::new();
    for field in fields {
        packet.extend_from_slice(&(field.len() as u64).to_le_bytes());
        packet.extend_from_slice(field.as_bytes());
    }
    std::fs::write(directory.join(cstr!("{id}.bin").as_str()), packet)
        .expect("retain complete BEFORE/expected/actual AFTER");
    assert_eq!(
        hits,
        vec![1; changes.len()],
        "each authored mutation occurs once"
    );
    assert_eq!(
        actual_errors,
        historical_errors.as_str(),
        "every original diagnostic field: {source}"
    );
    assert_eq!(
        actual_tree,
        expected_tree.as_str(),
        "every authored AFTER field: {source}"
    );
    historical_tree
}

#[cfg(feature = "legacy")]
pub(super) fn named_frozen_options_before(
    index: usize,
    source: &str,
    document: bool,
    options: &ParserOptions,
    tree: &str,
    errors: &str,
) -> Option<String> {
    let (required_source, in_tag_comments, change) = match index {
        6 => (
            "<p v-pre title='&fjlig;'>&fjlig; {{x}}</p>",
            false,
            Change {
                directive: false,
                name: "title",
                before: "f",
                after: "fj",
            },
        ),
        9 => (
            "<X // comment\n :title='&acE;'/>",
            true,
            Change {
                directive: true,
                name: "bind",
                before: "∾",
                after: "∾\u{333}",
            },
        ),
        _ => return None,
    };
    assert_eq!(source, required_source, "original frozen source");
    assert!(!document, "original frozen Component profile");
    assert_original_options(options, in_tag_comments);
    Some(prove_named_after(
        cstr!("frozen-option-{index}").as_str(),
        source,
        document,
        options,
        tree,
        errors,
        &[change],
    ))
}

#[cfg(feature = "legacy")]
fn assert_original_options(options: &ParserOptions, expected_in_tag_comments: bool) {
    // Exhaustive fields refuse future option growth without explicit custody.
    let ParserOptions {
        mode,
        whitespace,
        delimiters,
        is_pre_tag,
        is_native_tag,
        is_custom_element,
        custom_renderer,
        is_void_tag,
        get_namespace,
        on_error,
        on_warn,
        comments,
        experimental_in_tag_comments,
        dialect,
    } = options;
    let defaults = ParserOptions::default();
    assert_eq!(*mode, defaults.mode);
    assert_eq!(*whitespace, defaults.whitespace);
    assert_eq!(delimiters, &defaults.delimiters);
    assert!(core::ptr::fn_addr_eq(*is_pre_tag, defaults.is_pre_tag));
    assert!(is_native_tag.is_none());
    assert!(is_custom_element.is_none());
    assert!(!*custom_renderer);
    assert!(core::ptr::fn_addr_eq(*is_void_tag, defaults.is_void_tag));
    assert!(core::ptr::fn_addr_eq(
        *get_namespace,
        defaults.get_namespace
    ));
    assert!(on_error.is_none() && on_warn.is_none());
    assert!(*comments);
    assert_eq!(*experimental_in_tag_comments, expected_in_tag_comments);
    assert_eq!(*dialect, defaults.dialect);
}

pub(super) mod tests {
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
                    source
                        .get(expression.loc.span.start as usize..expression.loc.span.end as usize),
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
}
