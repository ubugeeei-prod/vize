//! Tests for the Vue template parser.

use super::{parse, parse_with_options};
use vize_carton::Bump;
use vize_relief::{
    ast::{ElementType, ExpressionNode, PropNode, TemplateChildNode},
    errors::ErrorCode,
    options::ParserOptions,
};

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
    assert_eq!(combined, "&lt;foo&gt;");
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
                "Hello &quot;World&quot;"
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
            assert_eq!(attr.value.as_ref().unwrap().content.as_str(), "&quot;");
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
    if let TemplateChildNode::Element(el) = &root.children[0] {
        if let PropNode::Directive(dir) = &el.props[0] {
            assert_eq!(dir.name.as_str(), "bind");
            if let Some(ExpressionNode::Simple(arg)) = &dir.arg {
                assert_eq!(arg.content.as_str(), "attr");
                assert!(!arg.is_static); // dynamic args are not static
            } else {
                panic!("Expected arg");
            }
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
        "<div><Foo />\n<!-- gap -->\n<div /></div>",
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
    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, r#"<div id="a" id="b"></div>"#);
    // Parser doesn't error on duplicate attrs, it just adds both
    // Verify both are present
    if let TemplateChildNode::Element(el) = &root.children[0] {
        assert_eq!(el.props.len(), 2);
    }
    let _ = errors;
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
