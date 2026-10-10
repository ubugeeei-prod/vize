//! Literal names preserve the opening ordinal and complete authored spans.

use super::{Parser, parse, parse_with_options_and_template_syntax};
use vize_l0::Allocator;
use vize_relief::options::{CustomElementMatcher, ParserOptions, TemplateSyntaxMode};
use vize_relief::{ElementType, PropNode, TemplateChildNode, errors::ErrorCode};

#[test]
fn complete_names_and_spans_follow_the_v_pre_boundary() {
    let source = "<div v-bind:[keys['name]']].camel.stop=\"before\" id=\"plain\" v-pre v-bind:[keys['name]']].camel.stop=\"after\"><Child is=\"vue:Child\" :[keys['name]']].camel=\"literal\"></Child></div>";
    let allocator = Allocator::new();
    let (root, errors, _) = Parser::new(&allocator, source).parse_with_frozen_elements();
    assert!(errors.is_empty(), "{errors:?}");
    let Some(TemplateChildNode::Element(div)) = root.children.first() else {
        panic!("expected complete div");
    };
    assert_eq!(div.tag_type, ElementType::Element);
    let names: Vec<_> = div
        .props
        .iter()
        .map(|prop| match prop {
            PropNode::Attribute(attr) => (attr.name, attr.name_loc.span.slice(source)),
            PropNode::Directive(_) => panic!("v-pre directive remained semantic"),
        })
        .collect();
    assert_eq!(
        names,
        [
            (
                "v-bind:[keys['name]']].camel.stop",
                "v-bind:[keys['name]']].camel.stop"
            ),
            ("id", "id"),
            (
                "v-bind[keys['name]']].camel.stop",
                "v-bind:[keys['name]']].camel.stop"
            ),
        ]
    );
    let Some(TemplateChildNode::Element(child)) = div.children.first() else {
        panic!("expected complete literal Child");
    };
    assert_eq!(child.tag_type, ElementType::Element);
    let names: Vec<_> = child
        .props
        .iter()
        .map(|prop| match prop {
            PropNode::Attribute(attr) => (attr.name, attr.name_loc.span.slice(source)),
            PropNode::Directive(_) => panic!("inherited v-pre name remained semantic"),
        })
        .collect();
    assert_eq!(
        names,
        [
            ("is", "is"),
            (":[keys['name]']].camel", ":[keys['name]']].camel")
        ]
    );
    for element in [div, child] {
        for prop in &element.props {
            let PropNode::Attribute(attr) = prop else {
                panic!("literal attribute");
            };
            let value = attr.value.as_ref().expect("valued original attribute");
            assert!(attr.loc.span.slice(source).ends_with('"'));
            assert!(attr.loc.span.slice(source).contains(value.content));
        }
    }
}

#[test]
fn inherited_literal_names_keep_real_missing_bracket_recovery() {
    for source in [
        "<div v-pre><span :[unterminated=\"literal\"></span></div>",
        "<div><span :[unterminated=\"literal\"></span></div>",
    ] {
        let allocator = Allocator::new();
        let (_, errors, _) = Parser::new(&allocator, source).parse_with_frozen_elements();
        assert_eq!(
            errors.iter().map(|error| error.code).collect::<Vec<_>>(),
            [ErrorCode::MissingDynamicDirectiveArgumentEnd],
            "{source}: {errors:?}"
        );
    }
}

#[test]
fn inherited_empty_modifier_is_literal_and_normal_negative_is_preserved() {
    for (source, expected) in [
        (
            "<div v-pre><span v-bind:id..camel=\"literal\"></span></div>",
            vec![],
        ),
        (
            "<div><span v-bind:id..camel=\"literal\"></span></div>",
            vec![ErrorCode::MissingDirectiveModifier],
        ),
    ] {
        let allocator = Allocator::new();
        let (_, errors) = parse(&allocator, source);
        assert_eq!(
            errors.iter().map(|error| error.code).collect::<Vec<_>>(),
            expected
        );
    }
}

#[test]
fn active_directives_keep_their_original_prefix_only_raw_names() {
    let source = "<Child :id=\"value\" #default=\"props\" v-bind:title=\"text\"></Child>";
    let allocator = Allocator::new();
    let (root, errors) = parse(&allocator, source);
    assert!(errors.is_empty(), "{errors:?}");
    let Some(TemplateChildNode::Element(child)) = root.children.first() else {
        panic!("Child");
    };
    assert_eq!(child.tag_type, ElementType::Component);
    let prefixes: Vec<_> = child
        .props
        .iter()
        .map(|prop| match prop {
            PropNode::Directive(dir) => dir.raw_name,
            PropNode::Attribute(_) => panic!("expected original active directive"),
        })
        .collect();
    assert_eq!(prefixes, [Some(":"), Some("#"), Some("v-bind")]);
}

#[test]
fn newly_literal_component_slashes_keep_their_flag_in_all_syntax_modes() {
    for mode in [
        TemplateSyntaxMode::Standard,
        TemplateSyntaxMode::Strict,
        TemplateSyntaxMode::Quirks,
    ] {
        for source in [
            "<Child v-pre is=\"vue:Child\"/>",
            "<div v-pre><Child is=\"vue:Child\"/></div>",
        ] {
            let arena = Allocator::new();
            let (root, errors, _) = Parser::with_options_and_template_syntax(
                &arena,
                source,
                ParserOptions::default(),
                mode,
            )
            .parse_with_frozen_elements();
            assert!(errors.is_empty(), "{source} {mode:?}: {errors:?}");
            let TemplateChildNode::Element(el) = &root.children[0] else {
                panic!("literal owner")
            };
            let child = if el.tag == "Child" {
                &**el
            } else {
                let TemplateChildNode::Element(child) = &el.children[0] else {
                    panic!("literal child")
                };
                &**child
            };
            assert_eq!(child.tag_type, ElementType::Element);
            assert!(child.is_self_closing);
            assert!(child.props.iter().any(|prop| matches!(prop, PropNode::Attribute(attr) if attr.name == "is" && attr.value.as_ref().is_some_and(|v| v.content == "vue:Child"))));
        }
    }
}

#[test]
fn native_unknown_and_custom_element_slashes_keep_existing_recovery_modes() {
    for mode in [
        TemplateSyntaxMode::Standard,
        TemplateSyntaxMode::Strict,
        TemplateSyntaxMode::Quirks,
    ] {
        for (tag, frozen, custom) in [
            ("p", false, false),
            ("p", true, false),
            ("tr", true, false),
            ("primitive", true, false),
            ("Child", false, true),
            ("Child", true, true),
        ] {
            let arena = Allocator::new();
            let source = vize_l0::cstr!(
                "<{tag}{}{}/>",
                if frozen { " v-pre" } else { "" },
                if tag == "tr" { "" } else { " is=\"vue:Child\"" }
            );
            let (root, errors) = parse_with_options_and_template_syntax(
                &arena,
                &source,
                ParserOptions {
                    is_custom_element: custom.then_some(|name| name == "Child"),
                    ..Default::default()
                },
                mode,
            );
            let TemplateChildNode::Element(el) = &root.children[0] else {
                panic!("authored element")
            };
            assert_eq!(el.tag_type, ElementType::Element);
            assert_eq!(el.is_self_closing, mode == TemplateSyntaxMode::Quirks);
            let expected_code = match mode {
                TemplateSyntaxMode::Standard => Some(ErrorCode::ExtendPoint),
                TemplateSyntaxMode::Strict => Some(ErrorCode::UnexpectedSolidusInTag),
                TemplateSyntaxMode::Quirks => None,
                _ => unreachable!(),
            };
            assert_eq!(
                errors.iter().map(|e| e.code).collect::<Vec<_>>(),
                expected_code.into_iter().collect::<Vec<_>>(),
                "{source} {mode:?}: {errors:?}"
            );
            if let Some(error) = errors.first() {
                assert_eq!(error.is_recoverable(), mode == TemplateSyntaxMode::Standard);
                assert_eq!(
                    error.loc.as_ref().map(|loc| loc.span.slice(&source)),
                    Some(source.as_str())
                );
            }
        }
    }
}

#[test]
fn normal_component_custom_renderer_and_declarative_custom_slash_policies_stay_distinct() {
    let arena = Allocator::new();
    let (root, errors) = parse(&arena, "<Child/>");
    assert!(errors.is_empty(), "{errors:?}");
    assert!(
        matches!(&root.children[0], TemplateChildNode::Element(el) if el.tag_type == ElementType::Component && el.is_self_closing)
    );
    let (root, errors) = super::parse_with_options(
        &arena,
        "<primitive v-pre/>",
        ParserOptions {
            custom_renderer: true,
            ..Default::default()
        },
    );
    assert!(errors.is_empty(), "{errors:?}");
    assert!(
        matches!(&root.children[0], TemplateChildNode::Element(el) if el.tag_type == ElementType::Element && el.is_self_closing)
    );
    let (root, errors) = Parser::with_options_custom_elements_and_template_syntax(
        &arena,
        "<Child v-pre/>",
        ParserOptions::default(),
        CustomElementMatcher::from_patterns(vec!["Child".into()]),
        TemplateSyntaxMode::Standard,
    )
    .parse();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].code, ErrorCode::ExtendPoint);
    assert!(
        matches!(&root.children[0], TemplateChildNode::Element(el) if el.tag_type == ElementType::Element && !el.is_self_closing)
    );
}

#[test]
fn missing_or_foreign_completed_heads_are_fatal_before_freezing_an_opening() {
    for (extra, message) in [
        (
            false,
            "Completed directive head custody is missing while freezing v-pre.",
        ),
        (
            true,
            "Completed directive head custody is missing while freezing v-pre.",
        ),
    ] {
        let arena = Allocator::new();
        let mut parser = Parser::new(&arena, "<div v-pre>");
        parser.frozen_elements = Some(vize_l0::Vec::new_in(&parser.allocator));
        // Feed the real private parser callbacks, then corrupt only custody.
        parser.on_open_tag_name_impl(1, 4);
        parser.on_dir_name_impl(5, 10);
        parser.on_attrib_name_end_impl(10);
        parser.on_attrib_end_impl(crate::tokenizer::QuoteType::NoValue, 10);
        let current = parser.current_element.as_mut().expect("actual opening");
        let Some(PropNode::Directive(dir)) = current.props.first_mut() else {
            panic!("completed v-pre")
        };
        dir.raw_name = if extra {
            Some(arena.alloc_str("v-pre"))
        } else {
            None
        };
        parser.on_open_tag_end_impl(10);
        let (root, errors) = parser.into_result();
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert_eq!(errors[0].code, ErrorCode::MissingDirectiveName);
        assert_eq!(errors[0].message, message);
        assert!(!errors[0].is_recoverable());
        assert!(root.children.is_empty());
    }
}
