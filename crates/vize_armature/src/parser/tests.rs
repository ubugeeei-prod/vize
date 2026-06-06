//! Tests for the Vue template parser.
#![allow(clippy::disallowed_macros)]

use super::{parse, parse_with_options, parse_with_options_and_template_syntax};
use vize_carton::Bump;
use vize_relief::{
    ast::{ElementType, ExpressionNode, Namespace, PropNode, TemplateChildNode},
    errors::{CompilerError, ErrorCode},
    options::{ParserOptions, TemplateSyntaxMode},
};

fn error_recovery_snapshot(errors: &[CompilerError]) -> std::vec::Vec<(ErrorCode, &str, &str)> {
    errors
        .iter()
        .map(|error| {
            (
                error.code,
                error.message.as_str(),
                error.loc.as_ref().map_or("", |loc| loc.source.as_str()),
            )
        })
        .collect()
}

#[test]
fn test_parse_simple_element() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<div></div>");

    assert!(errors.is_empty());
    assert_eq!(root.children.len(), 1);

    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert_eq!(el.tag.as_str(), "div");
        assert!(!el.is_self_closing);
    } else {
        panic!("Expected element node");
    }
}

#[test]
fn test_parse_text() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "hello");

    assert!(errors.is_empty());
    assert_eq!(root.children.len(), 1);

    if let TemplateChildNode::Text(text) = &root.children[0] {
        assert_eq!(text.content.as_str(), "hello");
    } else {
        panic!("Expected text node");
    }
}

#[test]
fn test_parse_condense_whitespace_collapses_runs_inside_text_nodes() {
    // Regression for #960: `whitespace: 'condense'` (the default) must
    // collapse runs of `[ \t\n\f\r]` to a single U+0020 inside text
    // nodes, matching `@vue/compiler-sfc`. The previous behavior left
    // mixed text nodes verbatim, so `x   y\n   z` stayed raw.
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<div>x   y\n   z</div>");
    assert!(errors.is_empty(), "{errors:?}");

    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert_eq!(el.children.len(), 1);
        if let TemplateChildNode::Text(text) = &el.children[0] {
            assert_eq!(text.content.as_str(), "x y z");
        } else {
            panic!("expected text child, got {:?}", el.children[0]);
        }
    } else {
        panic!("expected element root");
    }
}

#[test]
fn test_parse_text_with_entities_preserves_raw_source() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "&lt;foo&gt;");

    assert!(errors.is_empty());
    let combined = root
        .children
        .iter()
        .filter_map(|child| match child {
            TemplateChildNode::Text(text) => Some(text.content.as_str()),
            _ => None,
        })
        .collect::<std::vec::Vec<_>>()
        .join("");
    assert_eq!(combined, "<foo>");
}

#[test]
fn test_parse_interpolation() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "{{ msg }}");

    assert!(errors.is_empty());
    assert_eq!(root.children.len(), 1);

    if let TemplateChildNode::Interpolation(interp) = &root.children[0] {
        if let ExpressionNode::Simple(expr) = &interp.content {
            assert_eq!(expr.content.as_str(), "msg");
        } else {
            panic!("Expected simple expression");
        }
    } else {
        panic!("Expected interpolation node");
    }
}

#[test]
fn test_parse_directive() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, r#"<div v-if="ok"></div>"#);

    assert!(errors.is_empty());
    assert_eq!(root.children.len(), 1);

    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert_eq!(el.props.len(), 1);
        if let PropNode::Directive(dir) = &el.props[0] {
            assert_eq!(dir.name.as_str(), "if");
            if let Some(ExpressionNode::Simple(exp)) = &dir.exp {
                assert_eq!(exp.content.as_str(), "ok");
            }
        } else {
            panic!("Expected directive");
        }
    } else {
        panic!("Expected element node");
    }
}

#[test]
fn test_parse_shorthand_bind() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, r#"<div :class="cls"></div>"#);

    assert!(errors.is_empty());

    if let TemplateChildNode::Element(el) = &root.children[0] {
        if let PropNode::Directive(dir) = &el.props[0] {
            assert_eq!(dir.name.as_str(), "bind");
            if let Some(ExpressionNode::Simple(arg)) = &dir.arg {
                assert_eq!(arg.content.as_str(), "class");
            }
        } else {
            panic!("Expected directive");
        }
    }
}

#[test]
fn test_parse_shorthand_on() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, r#"<button @click="handler"></button>"#);

    assert!(errors.is_empty());

    if let TemplateChildNode::Element(el) = &root.children[0] {
        if let PropNode::Directive(dir) = &el.props[0] {
            assert_eq!(dir.name.as_str(), "on");
            if let Some(ExpressionNode::Simple(arg)) = &dir.arg {
                assert_eq!(arg.content.as_str(), "click");
            }
        } else {
            panic!("Expected directive");
        }
    }
}

#[test]
fn test_parse_nested_elements() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<div><span>text</span></div>");

    assert!(errors.is_empty());
    assert_eq!(root.children.len(), 1);

    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert_eq!(el.tag.as_str(), "div");
        assert_eq!(el.children.len(), 1);

        if let TemplateChildNode::Element(span) = &el.children[0] {
            assert_eq!(span.tag.as_str(), "span");
        }
    }
}

#[test]
fn test_parse_self_closing() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<input />");

    assert!(errors.is_empty());
    assert_eq!(root.children.len(), 1);

    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert_eq!(el.tag.as_str(), "input");
        assert!(el.is_self_closing);
    }
}

#[test]
fn test_parse_self_closing_textarea_warns_and_rewrites() {
    let allocator = Bump::new();
    let source = r#"<Primitive :class="ui.root({ class: [uiProp?.root, props.class] })"><textarea :class="ui.base({ class: uiProp?.base })" /><slot :ui="ui" /><span v-if="isLeading || !!avatar || !!slots.leading"><slot><UIcon v-if="isLeading && leadingIconName" /><UAvatar v-else-if="!!avatar" /></slot></span></Primitive>"#;
    let (root, errors) = parse(&allocator, source);

    assert!(
        errors.iter().any(|e| e.code == ErrorCode::ExtendPoint
            && e.message
                .contains("Invalid self-closing syntax on non-void HTML element")),
        "unexpected errors: {errors:?}"
    );
    assert!(errors.iter().all(CompilerError::is_recoverable));
    assert_eq!(root.children.len(), 1);

    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert_eq!(el.tag.as_str(), "Primitive");
        assert_eq!(el.children.len(), 3);

        if let TemplateChildNode::Element(textarea) = &el.children[0] {
            assert_eq!(textarea.tag.as_str(), "textarea");
            assert!(!textarea.is_self_closing);
            assert!(textarea.children.is_empty());
        } else {
            panic!("Expected textarea element");
        }
        assert!(
            matches!(&el.children[1], TemplateChildNode::Element(slot) if slot.tag.as_str() == "slot")
        );
        assert!(
            matches!(&el.children[2], TemplateChildNode::Element(span) if span.tag.as_str() == "span")
        );
    } else {
        panic!("Expected Primitive element");
    }
}

// ====================================================================
// Additional tests
// ====================================================================

#[test]
fn test_parse_comment() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<!-- hello -->");
    assert!(errors.is_empty());
    assert_eq!(root.children.len(), 1);
    if let TemplateChildNode::Comment(c) = &root.children[0] {
        assert_eq!(c.content.as_str(), " hello ");
    } else {
        panic!("Expected comment node");
    }
}

fn parser_options_svg_subtree() -> ParserOptions {
    ParserOptions {
        get_namespace: |tag, parent| {
            if tag.eq_ignore_ascii_case("svg")
                || parent.is_some_and(|p| p.eq_ignore_ascii_case("svg"))
            {
                Namespace::Svg
            } else {
                Namespace::Html
            }
        },
        ..ParserOptions::default()
    }
}

#[test]
fn test_parse_cdata_in_html_root_emits_error() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<![CDATA[hi]]>");
    assert!(root.children.is_empty());
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, ErrorCode::CdataInHtmlContent);
    assert_eq!(errors[0].loc.as_ref().unwrap().start.offset, 0);
}

#[test]
fn test_parse_cdata_in_html_element_emits_error() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<div><![CDATA[hi]]></div>");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, ErrorCode::CdataInHtmlContent);
    assert!(matches!(&root.children[0], TemplateChildNode::Element(_)));
    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert!(
            el.children.is_empty(),
            "CDATA must not become text in HTML ns"
        );
    }
}

#[test]
fn test_parse_cdata_in_svg_as_text() {
    let allocator = Bump::new();
    let (root, errors) = parse_with_options(
        &allocator,
        "<svg><![CDATA[hi]]></svg>",
        parser_options_svg_subtree(),
    );
    assert!(errors.is_empty());
    assert_eq!(root.children.len(), 1);
    if let TemplateChildNode::Element(svg) = &root.children[0] {
        assert_eq!(svg.tag.as_str(), "svg");
        assert_eq!(svg.ns, Namespace::Svg);
        assert_eq!(svg.children.len(), 1);
        if let TemplateChildNode::Text(t) = &svg.children[0] {
            assert_eq!(t.content.as_str(), "hi");
        } else {
            panic!("expected text node for CDATA body");
        }
    } else {
        panic!("expected svg element");
    }
}

#[test]
fn test_parse_void_element() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<input>");
    assert!(errors.is_empty());
    assert_eq!(root.children.len(), 1);
    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert_eq!(el.tag.as_str(), "input");
    } else {
        panic!("Expected element node");
    }
}

#[test]
fn test_parse_multiple_root_children() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<div></div><span></span>");
    assert!(errors.is_empty());
    assert_eq!(root.children.len(), 2);
    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert_eq!(el.tag.as_str(), "div");
    }
    if let TemplateChildNode::Element(el) = &root.children[1] {
        assert_eq!(el.tag.as_str(), "span");
    }
}

#[test]
fn test_parse_attribute_with_value() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, r#"<div id="foo"></div>"#);
    assert!(errors.is_empty());
    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert_eq!(el.props.len(), 1);
        if let PropNode::Attribute(attr) = &el.props[0] {
            assert_eq!(attr.name.as_str(), "id");
            assert_eq!(attr.value.as_ref().unwrap().content.as_str(), "foo");
        } else {
            panic!("Expected attribute");
        }
    }
}

#[test]
fn test_parse_attribute_with_trailing_entity() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, r#"<div title="Hello &quot;World&quot;"></div>"#);
    assert!(errors.is_empty());
    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert_eq!(el.props.len(), 1);
        if let PropNode::Attribute(attr) = &el.props[0] {
            assert_eq!(attr.name.as_str(), "title");
            assert_eq!(
                attr.value.as_ref().unwrap().content.as_str(),
                "Hello \"World\""
            );
        } else {
            panic!("Expected attribute");
        }
    }
}

#[test]
fn test_parse_attribute_value_with_only_entity() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, r#"<div title="&quot;"></div>"#);
    assert!(errors.is_empty());
    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert_eq!(el.props.len(), 1);
        if let PropNode::Attribute(attr) = &el.props[0] {
            assert_eq!(attr.name.as_str(), "title");
            assert_eq!(attr.value.as_ref().unwrap().content.as_str(), "\"");
        } else {
            panic!("Expected attribute");
        }
    }
}

#[test]
fn test_parse_boolean_attribute() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<input disabled>");
    assert!(errors.is_empty());
    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert_eq!(el.props.len(), 1);
        if let PropNode::Attribute(attr) = &el.props[0] {
            assert_eq!(attr.name.as_str(), "disabled");
            assert!(attr.value.is_none());
        } else {
            panic!("Expected attribute");
        }
    }
}

#[test]
fn test_parse_directive_modifiers() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, r#"<div @click.stop.prevent="h"></div>"#);
    assert!(errors.is_empty());
    if let TemplateChildNode::Element(el) = &root.children[0] {
        if let PropNode::Directive(dir) = &el.props[0] {
            assert_eq!(dir.name.as_str(), "on");
            assert_eq!(dir.modifiers.len(), 2);
            assert_eq!(dir.modifiers[0].content.as_str(), "stop");
            assert_eq!(dir.modifiers[1].content.as_str(), "prevent");
        } else {
            panic!("Expected directive");
        }
    }
}

#[test]
fn test_parse_dynamic_directive_arg() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, r#"<div v-bind:[attr]="val"></div>"#);
    assert!(errors.is_empty());
    if let TemplateChildNode::Element(el) = &root.children[0]
        && let PropNode::Directive(dir) = &el.props[0]
    {
        assert_eq!(dir.name.as_str(), "bind");
        if let Some(ExpressionNode::Simple(arg)) = &dir.arg {
            assert_eq!(arg.content.as_str(), "attr");
            assert!(!arg.is_static); // dynamic args are not static
        } else {
            panic!("Expected arg");
        }
    }
}

#[test]
fn test_parse_shorthand_slot() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<template #default></template>");
    assert!(errors.is_empty());
    if let TemplateChildNode::Element(el) = &root.children[0] {
        if let PropNode::Directive(dir) = &el.props[0] {
            assert_eq!(dir.name.as_str(), "slot");
            if let Some(ExpressionNode::Simple(arg)) = &dir.arg {
                assert_eq!(arg.content.as_str(), "default");
            }
        } else {
            panic!("Expected directive");
        }
    }
}

#[test]
fn test_parse_v_for() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, r#"<div v-for="item in items"></div>"#);
    assert!(errors.is_empty());
    if let TemplateChildNode::Element(el) = &root.children[0] {
        if let PropNode::Directive(dir) = &el.props[0] {
            assert_eq!(dir.name.as_str(), "for");
            if let Some(ExpressionNode::Simple(exp)) = &dir.exp {
                assert_eq!(exp.content.as_str(), "item in items");
            }
        } else {
            panic!("Expected directive");
        }
    }
}

#[test]
fn test_no_value_directive_loc_excludes_trailing_whitespace() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<input v-if />");
    assert!(errors.is_empty());

    if let TemplateChildNode::Element(el) = &root.children[0] {
        if let PropNode::Directive(dir) = &el.props[0] {
            assert_eq!(dir.loc.source.as_str(), "v-if");
        } else {
            panic!("Expected directive");
        }
    }
}

#[test]
fn test_quoted_attribute_loc_includes_closing_quote_with_spaced_equals() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, r#"<input class ="w-100" />"#);
    assert!(errors.is_empty());

    if let TemplateChildNode::Element(el) = &root.children[0] {
        if let PropNode::Attribute(attr) = &el.props[0] {
            assert_eq!(attr.loc.source.as_str(), r#"class ="w-100""#);
            assert_eq!(attr.value.as_ref().unwrap().content.as_str(), "w-100");
            assert_eq!(attr.value.as_ref().unwrap().loc.source.as_str(), "w-100");
        } else {
            panic!("Expected attribute");
        }
    }
}

#[test]
fn test_quoted_directive_loc_includes_closing_quote_with_spaced_equals() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, r#"<input v-if ="ok" />"#);
    assert!(errors.is_empty());

    if let TemplateChildNode::Element(el) = &root.children[0] {
        if let PropNode::Directive(dir) = &el.props[0] {
            assert_eq!(dir.loc.source.as_str(), r#"v-if ="ok""#);
            if let Some(ExpressionNode::Simple(exp)) = &dir.exp {
                assert_eq!(exp.content.as_str(), "ok");
                assert_eq!(exp.loc.source.as_str(), "ok");
            } else {
                panic!("Expected expression");
            }
        } else {
            panic!("Expected directive");
        }
    }
}

#[test]
fn test_parse_mixed_children() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<div>text<span></span>{{ msg }}</div>");
    assert!(errors.is_empty());
    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert_eq!(el.children.len(), 3);
        assert!(matches!(&el.children[0], TemplateChildNode::Text(_)));
        assert!(matches!(&el.children[1], TemplateChildNode::Element(_)));
        assert!(matches!(
            &el.children[2],
            TemplateChildNode::Interpolation(_)
        ));
    }
}

#[test]
fn test_parse_whitespace_condense() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<div>  <span></span>  </div>");
    assert!(errors.is_empty());
    if let TemplateChildNode::Element(el) = &root.children[0] {
        // Whitespace-only text nodes between elements with no newline are condensed to space
        assert!(el.children.len() <= 3);
    }
}

#[test]
fn test_parse_whitespace_condense_skips_comment_gaps_when_comments_disabled() {
    let allocator = Bump::new();
    let (root, errors) = parse_with_options(
        &allocator,
        "<div><Foo />\n<!-- gap -->\n<input /></div>",
        ParserOptions {
            comments: false,
            ..ParserOptions::default()
        },
    );
    assert!(errors.is_empty());
    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert_eq!(
            el.children.len(),
            2,
            "whitespace-only runs left behind after stripping comments should be removed",
        );
        assert!(matches!(&el.children[0], TemplateChildNode::Element(_)));
        assert!(matches!(&el.children[1], TemplateChildNode::Element(_)));
    }
}

#[test]
fn test_parse_whitespace_condense_preserves_pre_children() {
    let allocator = Bump::new();
    let (root, errors) = parse_with_options(
        &allocator,
        "<pre>\n  hello\n  world\n</pre>",
        ParserOptions {
            is_pre_tag: |tag| tag == "pre",
            ..ParserOptions::default()
        },
    );
    assert!(errors.is_empty());
    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert_eq!(el.children.len(), 1);
        match &el.children[0] {
            TemplateChildNode::Text(text) => {
                assert_eq!(text.content.as_str(), "\n  hello\n  world\n");
            }
            _ => panic!("expected preserved text node"),
        }
    }
}

#[test]
fn test_parse_error_missing_end_tag() {
    let allocator = Bump::new();
    let (_root, errors) = parse(&allocator, "<div>");
    assert!(!errors.is_empty());
    assert!(errors.iter().any(|e| e.code == ErrorCode::MissingEndTag));
}

#[test]
fn test_parse_error_duplicate_attribute() {
    // A duplicate attribute is recorded as a recoverable diagnostic
    // (#958). Both occurrences remain in the AST so linters can warn
    // about the repeat; downstream codegen treats the diagnostic as
    // non-fatal and emits valid render code for the first occurrence.
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, r#"<div id="a" id="b"></div>"#);
    assert!(
        errors
            .iter()
            .any(|e| e.code == ErrorCode::DuplicateAttribute)
    );
    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert_eq!(el.props.len(), 2);
    }
}

#[test]
fn test_parse_error_duplicate_attribute_is_ascii_case_insensitive() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, r#"<div id="a" ID="b"></div>"#);

    assert!(
        errors
            .iter()
            .any(|e| e.code == ErrorCode::DuplicateAttribute)
    );
    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert_eq!(el.props.len(), 2);
    }
}

#[test]
fn test_parse_deep_nesting() {
    let allocator = Bump::new();
    let (root, errors) = parse(
        &allocator,
        "<div><span><p><em><strong>deep</strong></em></p></span></div>",
    );
    assert!(errors.is_empty());
    // Traverse 5 levels deep
    if let TemplateChildNode::Element(div) = &root.children[0] {
        assert_eq!(div.tag.as_str(), "div");
        if let TemplateChildNode::Element(span) = &div.children[0] {
            assert_eq!(span.tag.as_str(), "span");
            if let TemplateChildNode::Element(p) = &span.children[0] {
                assert_eq!(p.tag.as_str(), "p");
                if let TemplateChildNode::Element(em) = &p.children[0] {
                    assert_eq!(em.tag.as_str(), "em");
                    if let TemplateChildNode::Element(strong) = &em.children[0] {
                        assert_eq!(strong.tag.as_str(), "strong");
                    }
                }
            }
        }
    }
}

#[test]
fn test_parse_extreme_nesting_is_bounded() {
    // Pathologically deep input should parse without unbounded growth and
    // surface a recoverable error rather than producing an AST that later
    // passes would have to recurse into without limit.
    let allocator = Bump::new();
    let depth = 5000;
    let mut source = String::new();
    for _ in 0..depth {
        source.push_str("<div>");
    }
    for _ in 0..depth {
        source.push_str("</div>");
    }

    let (root, errors) = parse(&allocator, &source);

    // The nesting limit was reported.
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("nesting is too deep")),
        "expected a nesting-depth error for deeply nested input"
    );

    // The retained tree depth is bounded by the cap, not by the input depth.
    let mut node = root.children.first();
    let mut measured = 0usize;
    while let Some(TemplateChildNode::Element(el)) = node {
        measured += 1;
        node = el.children.first();
    }
    assert!(
        measured <= 257,
        "tree depth should stay bounded near the cap, got {measured}"
    );
}

#[test]
fn test_parse_component() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<MyComponent></MyComponent>");
    assert!(errors.is_empty());
    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert_eq!(el.tag.as_str(), "MyComponent");
        assert_eq!(el.tag_type, ElementType::Component);
    }
}

#[test]
fn test_empty_quoted_attribute_double() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, r#"<img alt="" />"#);
    assert!(errors.is_empty());
    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert_eq!(el.props.len(), 1);
        if let PropNode::Attribute(attr) = &el.props[0] {
            assert_eq!(attr.name.as_str(), "alt");
            let value = attr.value.as_ref().expect("alt=\"\" should have a value");
            assert_eq!(
                value.content.as_str(),
                "",
                "alt=\"\" should be empty string, not boolean"
            );
        } else {
            panic!("Expected attribute prop");
        }
    }
}

#[test]
fn test_empty_quoted_attribute_single() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<img alt='' />");
    assert!(errors.is_empty());
    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert_eq!(el.props.len(), 1);
        if let PropNode::Attribute(attr) = &el.props[0] {
            assert_eq!(attr.name.as_str(), "alt");
            let value = attr.value.as_ref().expect("alt='' should have a value");
            assert_eq!(
                value.content.as_str(),
                "",
                "alt='' should be empty string, not boolean"
            );
        } else {
            panic!("Expected attribute prop");
        }
    }
}

#[test]
fn test_empty_quoted_attribute_disabled() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, r#"<input disabled="" />"#);
    assert!(errors.is_empty());
    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert_eq!(el.props.len(), 1);
        if let PropNode::Attribute(attr) = &el.props[0] {
            assert_eq!(attr.name.as_str(), "disabled");
            let value = attr
                .value
                .as_ref()
                .expect("disabled=\"\" should have a value");
            assert_eq!(value.content.as_str(), "");
        } else {
            panic!("Expected attribute prop");
        }
    }
}

#[test]
fn test_parse_open_tag_at_eof_does_not_panic() {
    // Regression: a tag that is still open at EOF used to panic with
    // "byte index N is out of bounds" because on_error was called with
    // index == source.len(), and create_loc(index, index+1) sliced past the end.
    let allocator = Bump::new();
    let (_root, errors) = parse(&allocator, "<template>\n  <div>\n  <div\n");
    // Should return errors (EofInTag / MissingEndTag), not panic.
    assert!(!errors.is_empty());
}

#[test]
fn test_parse_malformed_close_tag_at_eof_does_not_panic() {
    let allocator = Bump::new();
    let (_root, errors) = parse(&allocator, "\n  <div class=\"root\">{{ title }}</di=\"{");

    assert!(
        errors
            .iter()
            .any(|error| error.code == ErrorCode::InvalidEndTag)
    );
}

#[test]
fn test_parse_recovers_open_tag_at_eof() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, r#"<div class="a""#);

    assert!(errors.iter().any(|e| e.code == ErrorCode::EofInTag));
    assert!(errors.iter().any(|e| e.code == ErrorCode::MissingEndTag));
    assert_eq!(root.children.len(), 1);

    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert_eq!(el.tag.as_str(), "div");
        if let PropNode::Attribute(attr) = &el.props[0] {
            assert_eq!(attr.name.as_str(), "class");
            assert_eq!(attr.value.as_ref().unwrap().content.as_str(), "a");
        } else {
            panic!("Expected recovered attribute");
        }
    } else {
        panic!("Expected recovered element");
    }
}

#[test]
fn test_parse_recovers_missing_attribute_value() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<div id=></div>");

    assert!(
        errors
            .iter()
            .any(|e| e.code == ErrorCode::MissingAttributeValue)
    );
    if let TemplateChildNode::Element(el) = &root.children[0] {
        if let PropNode::Attribute(attr) = &el.props[0] {
            assert_eq!(attr.name.as_str(), "id");
            assert_eq!(attr.value.as_ref().unwrap().content.as_str(), "");
        } else {
            panic!("Expected recovered attribute");
        }
    } else {
        panic!("Expected element");
    }
}

#[test]
fn test_parse_recovers_missing_equals_before_quoted_attribute_value() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, r#"<div id"foo"></div>"#);

    assert!(
        errors
            .iter()
            .any(|e| e.code == ErrorCode::UnexpectedCharacterInAttributeName)
    );
    if let TemplateChildNode::Element(el) = &root.children[0] {
        if let PropNode::Attribute(attr) = &el.props[0] {
            assert_eq!(attr.name.as_str(), "id");
            assert_eq!(attr.value.as_ref().unwrap().content.as_str(), "foo");
        } else {
            panic!("Expected recovered attribute");
        }
    } else {
        panic!("Expected element");
    }
}

#[test]
fn test_parse_reports_missing_whitespace_between_attributes_and_continues() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, r#"<div id="a"class="b"></div>"#);

    assert!(
        errors
            .iter()
            .any(|e| e.code == ErrorCode::MissingWhitespaceBetweenAttributes)
    );
    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert_eq!(el.props.len(), 2);
        if let PropNode::Attribute(attr) = &el.props[1] {
            assert_eq!(attr.name.as_str(), "class");
            assert_eq!(attr.value.as_ref().unwrap().content.as_str(), "b");
        } else {
            panic!("Expected recovered second attribute");
        }
    } else {
        panic!("Expected element");
    }
}

#[test]
fn test_parse_recovers_missing_dynamic_directive_argument_end() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, r#"<div :[foo="bar"></div>"#);

    assert!(
        errors
            .iter()
            .any(|e| e.code == ErrorCode::MissingDynamicDirectiveArgumentEnd)
    );
    if let TemplateChildNode::Element(el) = &root.children[0] {
        if let PropNode::Directive(dir) = &el.props[0] {
            assert_eq!(dir.name.as_str(), "bind");
            if let Some(ExpressionNode::Simple(arg)) = &dir.arg {
                assert_eq!(arg.content.as_str(), "foo");
                assert!(!arg.is_static);
            } else {
                panic!("Expected recovered directive argument");
            }
        } else {
            panic!("Expected recovered directive");
        }
    } else {
        panic!("Expected element");
    }
}

#[test]
fn test_parse_reports_missing_directive_name_and_continues() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<div v->ok</div>");

    assert!(
        errors
            .iter()
            .any(|e| e.code == ErrorCode::MissingDirectiveName)
    );
    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert!(el.props.is_empty());
        assert_eq!(el.children.len(), 1);
    } else {
        panic!("Expected element");
    }
}

#[test]
fn test_parse_unfinished_interpolation_reports_error_but_keeps_text() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "{{ unfinished");

    assert!(
        errors
            .iter()
            .any(|e| e.code == ErrorCode::MissingInterpolationEnd)
    );
    assert_eq!(root.children.len(), 1);
    if let TemplateChildNode::Text(text) = &root.children[0] {
        assert_eq!(text.content.as_str(), "{{ unfinished");
    } else {
        panic!("Expected recovered text");
    }
}

#[test]
fn test_parse_invalid_tag_name_reports_error_and_continues() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<1div></1div><span></span>");

    assert!(
        errors
            .iter()
            .any(|e| e.code == ErrorCode::InvalidFirstCharacterOfTagName)
    );
    assert!(
        root.children.iter().any(
            |child| matches!(child, TemplateChildNode::Element(el) if el.tag.as_str() == "span")
        )
    );
}

#[test]
fn test_parse_mixed_broken_input_keeps_later_nodes() {
    let allocator = Bump::new();
    let source = "<div v-><1bad></1bad><span id=a>ok</span></div>{{ broken";
    let (root, errors) = parse(&allocator, source);

    insta::assert_debug_snapshot!(error_recovery_snapshot(&errors), @r###"
    [
        (
            MissingDirectiveName,
            "Directive `v-` is missing a name. Ignoring it so the rest of the tag can be parsed.",
            "v-",
        ),
        (
            InvalidFirstCharacterOfTagName,
            "Tag name starts with an invalid character; treating the malformed tag as text.",
            "1",
        ),
        (
            InvalidFirstCharacterOfTagName,
            "Tag name starts with an invalid character; treating the malformed tag as text.",
            "1",
        ),
        (
            InvalidEndTag,
            "Invalid end tag.",
            "</1bad>",
        ),
        (
            MissingInterpolationEnd,
            "Interpolation is missing its closing delimiter `}}`; treating the unfinished interpolation as text.",
            "",
        ),
    ]
    "###);

    let TemplateChildNode::Element(div) = &root.children[0] else {
        panic!("Expected recovered div element");
    };
    assert_eq!(div.tag.as_str(), "div");
    assert!(
        div.children.iter().any(
            |child| matches!(child, TemplateChildNode::Element(el) if el.tag.as_str() == "span")
        )
    );
}

#[test]
fn test_parse_incorrectly_closed_comment_reports_error_and_continues() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<!-- note --!><div></div>");

    assert!(
        errors
            .iter()
            .any(|e| e.code == ErrorCode::IncorrectlyClosedComment)
    );
    assert_eq!(root.children.len(), 2);
    assert!(matches!(&root.children[0], TemplateChildNode::Comment(_)));
    assert!(matches!(&root.children[1], TemplateChildNode::Element(_)));
}

#[test]
fn test_parse_abrupt_empty_comment_reports_error_and_continues() {
    for comment_source in ["<!-->", "<!--->"] {
        let allocator = Bump::new();
        let source = format!("{comment_source}<div></div>");
        let (root, errors) = parse(&allocator, &source);

        assert!(
            errors
                .iter()
                .any(|e| e.code == ErrorCode::AbruptClosingOfEmptyComment)
        );
        assert_eq!(root.children.len(), 2);
        if let TemplateChildNode::Comment(comment) = &root.children[0] {
            assert_eq!(comment.content.as_str(), "");
            assert_eq!(comment.loc.source.as_str(), comment_source);
        } else {
            panic!("Expected recovered comment");
        }
        assert!(matches!(&root.children[1], TemplateChildNode::Element(_)));
    }
}

#[test]
fn test_parse_nested_comment_reports_error_and_closes_at_first_end() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<!-- <!-- nested --> -->");

    assert!(errors.iter().any(|e| e.code == ErrorCode::NestedComment));
    assert_eq!(root.children.len(), 2);
    if let TemplateChildNode::Comment(comment) = &root.children[0] {
        assert_eq!(comment.content.as_str(), " <!-- nested ");
    } else {
        panic!("Expected first node to be the recovered comment");
    }
    if let TemplateChildNode::Text(text) = &root.children[1] {
        assert_eq!(text.content.as_str(), " -->");
    } else {
        panic!("Expected trailing close marker text");
    }
}

#[test]
fn test_parse_processing_instruction_reports_error_and_continues() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, r#"<?xml version="1.0"?><div></div>"#);

    assert!(
        errors
            .iter()
            .any(|e| e.code == ErrorCode::UnexpectedQuestionMarkInsteadOfTagName)
    );
    assert!(
        root.children.iter().any(
            |child| matches!(child, TemplateChildNode::Element(el) if el.tag.as_str() == "div")
        )
    );
}

#[test]
fn test_parse_unexpected_solidus_before_attribute_reports_error_and_continues() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<div / id=foo></div>");

    assert!(
        errors
            .iter()
            .any(|e| e.code == ErrorCode::UnexpectedSolidusInTag)
    );
    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert!(!el.is_self_closing);
        if let PropNode::Attribute(attr) = &el.props[0] {
            assert_eq!(attr.name.as_str(), "id");
            assert_eq!(attr.value.as_ref().unwrap().content.as_str(), "foo");
        } else {
            panic!("Expected recovered attribute");
        }
    } else {
        panic!("Expected element");
    }
}

#[test]
fn test_parse_self_closing_non_void_html_element_warns_and_rewrites() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<div /><span></span>");

    assert!(errors.iter().any(|e| {
        e.code == ErrorCode::ExtendPoint
            && e.message
                .contains("Invalid self-closing syntax on non-void HTML element")
    }));
    assert!(errors.iter().all(CompilerError::is_recoverable));
    assert_eq!(root.children.len(), 2);
    if let TemplateChildNode::Element(div) = &root.children[0] {
        assert_eq!(div.tag.as_str(), "div");
        assert!(!div.is_self_closing);
        assert!(div.children.is_empty());
    } else {
        panic!("Expected div");
    }
    assert!(
        matches!(&root.children[1], TemplateChildNode::Element(span) if span.tag.as_str() == "span")
    );
}

#[test]
fn test_parse_self_closing_non_void_html_element_strict_errors_and_rewrites() {
    let allocator = Bump::new();
    let (root, errors) = parse_with_options_and_template_syntax(
        &allocator,
        "<div /><span></span>",
        ParserOptions::default(),
        TemplateSyntaxMode::Strict,
    );

    assert!(errors.iter().any(|e| {
        e.code == ErrorCode::UnexpectedSolidusInTag
            && e.message
                .contains("Invalid self-closing syntax on non-void HTML element")
    }));
    assert!(errors.iter().any(|e| !e.is_recoverable()));
    assert_eq!(root.children.len(), 2);
    if let TemplateChildNode::Element(div) = &root.children[0] {
        assert_eq!(div.tag.as_str(), "div");
        assert!(!div.is_self_closing);
        assert!(div.children.is_empty());
    } else {
        panic!("Expected div");
    }
    assert!(
        matches!(&root.children[1], TemplateChildNode::Element(span) if span.tag.as_str() == "span")
    );
}

#[test]
fn test_parse_self_closing_non_void_html_element_quirk_keeps_flag() {
    let allocator = Bump::new();
    let (root, errors) = parse_with_options_and_template_syntax(
        &allocator,
        "<div /><span></span>",
        ParserOptions::default(),
        TemplateSyntaxMode::Quirks,
    );

    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(root.children.len(), 2);
    if let TemplateChildNode::Element(div) = &root.children[0] {
        assert_eq!(div.tag.as_str(), "div");
        assert!(div.is_self_closing);
        assert!(div.children.is_empty());
    } else {
        panic!("Expected div");
    }
    assert!(
        matches!(&root.children[1], TemplateChildNode::Element(span) if span.tag.as_str() == "span")
    );
}

#[test]
fn test_parse_custom_renderer_non_html_element_keeps_self_closing_flag() {
    let allocator = Bump::new();
    let (root, errors) = parse_with_options(
        &allocator,
        "<primitive />",
        ParserOptions {
            custom_renderer: true,
            ..ParserOptions::default()
        },
    );

    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(root.children.len(), 1);
    if let TemplateChildNode::Element(primitive) = &root.children[0] {
        assert_eq!(primitive.tag.as_str(), "primitive");
        assert_eq!(primitive.tag_type, ElementType::Element);
        assert!(primitive.is_self_closing);
    } else {
        panic!("Expected primitive");
    }
}

#[test]
fn test_parse_table_inserts_implicit_tbody() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<table><tr><td>x</td></tr></table>");

    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    let TemplateChildNode::Element(table) = &root.children[0] else {
        panic!("Expected table");
    };
    assert_eq!(table.children.len(), 1);
    let TemplateChildNode::Element(tbody) = &table.children[0] else {
        panic!("Expected implicit tbody");
    };
    assert_eq!(tbody.tag.as_str(), "tbody");
    let TemplateChildNode::Element(tr) = &tbody.children[0] else {
        panic!("Expected tr");
    };
    let TemplateChildNode::Element(td) = &tr.children[0] else {
        panic!("Expected td");
    };
    assert!(
        matches!(&td.children[0], TemplateChildNode::Text(text) if text.content.as_str() == "x")
    );
}

#[test]
fn test_parse_table_foster_parents_unexpected_element() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<table><div>hello</div></table>");

    assert!(
        errors
            .iter()
            .any(|e| e.code == ErrorCode::ExtendPoint && e.message.contains("Foster parenting"))
    );
    assert_eq!(root.children.len(), 2);
    assert!(
        matches!(&root.children[0], TemplateChildNode::Element(div) if div.tag.as_str() == "div")
    );
    assert!(
        matches!(&root.children[1], TemplateChildNode::Element(table) if table.tag.as_str() == "table")
    );
}

#[test]
fn test_parse_table_cell_keeps_normal_body_content() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<table><tr><td><div>ok</div></td></tr></table>");

    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    let TemplateChildNode::Element(table) = &root.children[0] else {
        panic!("Expected table");
    };
    let TemplateChildNode::Element(tbody) = &table.children[0] else {
        panic!("Expected tbody");
    };
    let TemplateChildNode::Element(tr) = &tbody.children[0] else {
        panic!("Expected tr");
    };
    let TemplateChildNode::Element(td) = &tr.children[0] else {
        panic!("Expected td");
    };
    assert!(
        matches!(&td.children[0], TemplateChildNode::Element(div) if div.tag.as_str() == "div")
    );
}

#[test]
fn test_parse_adoption_agency_repairs_misnested_formatting() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<b>aaa<i>bbb</b>ccc</i>");

    assert!(
        errors
            .iter()
            .any(|e| e.code == ErrorCode::ExtendPoint && e.message.contains("adoption agency"))
    );
    assert_eq!(root.children.len(), 2);
    let TemplateChildNode::Element(b) = &root.children[0] else {
        panic!("Expected b");
    };
    assert_eq!(b.tag.as_str(), "b");
    assert!(
        matches!(&b.children[0], TemplateChildNode::Text(text) if text.content.as_str() == "aaa")
    );
    let TemplateChildNode::Element(inner_i) = &b.children[1] else {
        panic!("Expected nested i");
    };
    assert_eq!(inner_i.tag.as_str(), "i");
    assert!(
        matches!(&inner_i.children[0], TemplateChildNode::Text(text) if text.content.as_str() == "bbb")
    );

    let TemplateChildNode::Element(reopened_i) = &root.children[1] else {
        panic!("Expected reopened i");
    };
    assert_eq!(reopened_i.tag.as_str(), "i");
    assert!(
        matches!(&reopened_i.children[0], TemplateChildNode::Text(text) if text.content.as_str() == "ccc")
    );
}

#[test]
fn test_parse_in_body_omits_p_and_li_end_tags() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<p>a<p>b<ul><li>c<li>d</ul>");

    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(root.children.len(), 3);
    assert!(matches!(&root.children[0], TemplateChildNode::Element(p) if p.tag.as_str() == "p"));
    assert!(matches!(&root.children[1], TemplateChildNode::Element(p) if p.tag.as_str() == "p"));
    let TemplateChildNode::Element(ul) = &root.children[2] else {
        panic!("Expected ul");
    };
    assert_eq!(ul.children.len(), 2);
    assert!(matches!(&ul.children[0], TemplateChildNode::Element(li) if li.tag.as_str() == "li"));
    assert!(matches!(&ul.children[1], TemplateChildNode::Element(li) if li.tag.as_str() == "li"));
}

#[test]
fn test_parse_nested_anchor_and_button_are_split() {
    let allocator = Bump::new();
    let (anchor_root, anchor_errors) = parse(
        &allocator,
        r#"<a href="/">outer<a href="/foo">inner</a></a>"#,
    );

    assert!(
        anchor_errors
            .iter()
            .any(|e| e.code == ErrorCode::ExtendPoint && e.message.contains("Nested anchor"))
    );
    assert_eq!(anchor_root.children.len(), 2);
    assert!(
        matches!(&anchor_root.children[0], TemplateChildNode::Element(a) if a.tag.as_str() == "a")
    );
    assert!(
        matches!(&anchor_root.children[1], TemplateChildNode::Element(a) if a.tag.as_str() == "a")
    );

    let (button_root, button_errors) =
        parse(&allocator, "<button>aaa<button>bbb</button></button>");
    assert!(
        button_errors
            .iter()
            .any(|e| e.code == ErrorCode::ExtendPoint && e.message.contains("Nested button"))
    );
    assert_eq!(button_root.children.len(), 2);
    assert!(
        matches!(&button_root.children[0], TemplateChildNode::Element(button) if button.tag.as_str() == "button")
    );
    assert!(
        matches!(&button_root.children[1], TemplateChildNode::Element(button) if button.tag.as_str() == "button")
    );
}

#[test]
fn test_parse_nested_form_start_tag_is_ignored() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<form><input><form><input></form></form>");

    assert!(
        errors
            .iter()
            .any(|e| e.code == ErrorCode::ExtendPoint
                && e.message.contains("ignored this start tag"))
    );
    assert_eq!(root.children.len(), 1);
    let TemplateChildNode::Element(form) = &root.children[0] else {
        panic!("Expected form");
    };
    assert_eq!(form.tag.as_str(), "form");
    assert_eq!(form.children.len(), 2);
    assert!(form.children.iter().all(
        |child| matches!(child, TemplateChildNode::Element(input) if input.tag.as_str() == "input")
    ));
}

#[test]
fn test_parse_unclosed_comment_reports_error_without_losing_comment() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "before<!-- open");

    insta::assert_debug_snapshot!(error_recovery_snapshot(&errors), @r###"
    [
        (
            EofInComment,
            "Comment is missing its closing `-->`; preserving the unfinished comment so parsing can finish.",
            "<",
        ),
    ]
    "###);
    assert_eq!(root.children.len(), 2);
    assert!(matches!(&root.children[0], TemplateChildNode::Text(_)));
    if let TemplateChildNode::Comment(comment) = &root.children[1] {
        assert_eq!(comment.content.as_str(), " open");
        assert_eq!(comment.loc.source.as_str(), "<!-- open");
    } else {
        panic!("Expected recovered comment");
    }
}

#[test]
fn test_boolean_attribute_no_value() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<input disabled />");
    assert!(errors.is_empty());
    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert_eq!(el.props.len(), 1);
        if let PropNode::Attribute(attr) = &el.props[0] {
            assert_eq!(attr.name.as_str(), "disabled");
            assert!(
                attr.value.is_none(),
                "disabled without value should be boolean (None)"
            );
        } else {
            panic!("Expected attribute prop");
        }
    }
}

// ====================================================================
// HTML entity: consecutive text + entity merge into one Text node
// ====================================================================

#[test]
fn test_parse_text_entity_named_amp_between_literals() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<div>a&amp;b</div>");
    assert!(errors.is_empty());
    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert_eq!(el.children.len(), 1);
        if let TemplateChildNode::Text(t) = &el.children[0] {
            assert_eq!(t.content.as_str(), "a&b");
            assert_eq!(t.loc.source.as_str(), "a&amp;b");
        } else {
            panic!("expected text");
        }
    } else {
        panic!("expected element");
    }
}

#[test]
fn test_parse_text_entity_lt_only() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<div>&lt;</div>");
    assert!(errors.is_empty());
    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert_eq!(el.children.len(), 1);
        if let TemplateChildNode::Text(t) = &el.children[0] {
            assert_eq!(t.content.as_str(), "<");
        } else {
            panic!("expected text");
        }
    }
}

#[test]
fn test_parse_text_entity_numeric_dec() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<div>&#38;x</div>");
    assert!(errors.is_empty());
    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert_eq!(el.children.len(), 1);
        if let TemplateChildNode::Text(t) = &el.children[0] {
            assert_eq!(t.content.as_str(), "&x");
            assert_eq!(t.loc.source.as_str(), "&#38;x");
        } else {
            panic!("expected text");
        }
    }
}

#[test]
fn test_parse_text_entity_1_lt_2() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<div>1&lt;2</div>");
    assert!(errors.is_empty());
    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert_eq!(el.children.len(), 1);
        if let TemplateChildNode::Text(t) = &el.children[0] {
            assert_eq!(t.content.as_str(), "1<2");
            assert_eq!(t.loc.source.as_str(), "1&lt;2");
        } else {
            panic!("expected text");
        }
    }
}

#[test]
fn test_parse_attribute_entity_single_quoted_numeric() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, "<div a='&#38;'></div>");
    assert!(errors.is_empty());
    if let TemplateChildNode::Element(el) = &root.children[0] {
        if let PropNode::Attribute(attr) = &el.props[0] {
            assert_eq!(attr.value.as_ref().unwrap().content.as_str(), "&");
        } else {
            panic!("Expected attribute");
        }
    }
}

#[test]
fn test_parse_attribute_entity_quot_in_double_quotes() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, r#"<div class="a &quot; b"></div>"#);
    assert!(errors.is_empty());
    if let TemplateChildNode::Element(el) = &root.children[0] {
        if let PropNode::Attribute(attr) = &el.props[0] {
            assert_eq!(attr.name.as_str(), "class");
            assert_eq!(attr.value.as_ref().unwrap().content.as_str(), "a \" b");
        } else {
            panic!("Expected attribute");
        }
    }
}

#[test]
fn test_parse_attribute_entity_lt_gt() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, r#"<div title="&lt;tag&gt;"></div>"#);
    assert!(errors.is_empty());
    if let TemplateChildNode::Element(el) = &root.children[0] {
        if let PropNode::Attribute(attr) = &el.props[0] {
            assert_eq!(attr.value.as_ref().unwrap().content.as_str(), "<tag>");
        } else {
            panic!("Expected attribute");
        }
    }
}

#[test]
fn test_parse_directive_value_entity_is_decoded() {
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, r#"<div v-if="a&amp;&amp;b"></div>"#);
    assert!(errors.is_empty());

    if let TemplateChildNode::Element(el) = &root.children[0] {
        if let PropNode::Directive(dir) = &el.props[0] {
            if let Some(ExpressionNode::Simple(exp)) = &dir.exp {
                assert_eq!(exp.content.as_str(), "a&&b");
            } else {
                panic!("Expected simple expression");
            }
        } else {
            panic!("Expected directive");
        }
    }
}
