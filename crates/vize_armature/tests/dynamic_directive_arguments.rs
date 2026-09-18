use vize_armature::parse;
use vize_relief::{ErrorCode, ExpressionNode, PropNode, TemplateChildNode};
use vize_s0::{Allocator, cstr};

#[test]
fn nested_dynamic_arguments_preserve_content_spans_and_retained_ast() {
    for prefix in [":", "v-bind:", "@", "v-on:", "v-model:"] {
        for argument in [
            "state.names[state.index]",
            "state.names[indices[state.index]]",
            "[state.first,state.second][state.index]",
            "state.keys[']']",
            r#"state.keys["]"]"#,
            r#"state.keys['\']']"#,
            r#"state.keys['\\'][state.index]"#,
            "state.keys['\u{1f680}]']",
            "({'key]':state.field})['key]']",
            "`key${state.names[state.index]}`",
            "`key${`nested${state.keys[']']}`}`",
        ] {
            let source = cstr!("<Child\n {prefix}[{argument}].foo.bar=\"value\" id=\"tail\"/>");
            let allocator = Allocator::new();
            let (root, errors) = parse(&allocator, &source);
            assert!(errors.is_empty(), "{source}: {errors:?}");
            let TemplateChildNode::Element(element) = &root.children[0] else {
                panic!("expected component");
            };
            assert_eq!(element.props.len(), 2, "{source}");
            let PropNode::Directive(directive) = &element.props[0] else {
                panic!("expected directive");
            };
            let Some(ExpressionNode::Simple(arg)) = &directive.arg else {
                panic!("expected dynamic argument");
            };
            assert_eq!(arg.content, argument, "{source}");
            assert!(!arg.is_static, "{source}");
            let start = source.find(argument).unwrap();
            assert_eq!(arg.loc.span.start as usize, start);
            assert_eq!(arg.loc.span.end as usize, start + argument.len());
            assert_eq!(arg.js_ast.as_ref().expect("retained AST").raw, argument);
            assert_eq!(directive.modifiers.len(), 2);
            assert_eq!(directive.modifiers[0].content, "foo");
            assert_eq!(directive.modifiers[1].content, "bar");
            let PropNode::Attribute(tail) = &element.props[1] else {
                panic!("expected following attribute");
            };
            assert_eq!(tail.name, "id");
            assert_eq!(tail.value.as_ref().unwrap().content, "tail");
        }
    }
}

#[test]
fn incomplete_dynamic_arguments_recover_at_attribute_and_tag_boundaries() {
    for argument in [
        "state.names[state.index]",
        "state.names[state.index",
        "state.keys['broken",
    ] {
        let source = cstr!("<Child :[{argument}=\"value\" id=\"tail\"/><p>after</p>");
        let allocator = Allocator::new();
        let (root, errors) = parse(&allocator, &source);
        let boundary = source.find("=\"value\"").unwrap();
        let error = errors
            .iter()
            .find(|error| error.code == ErrorCode::MissingDynamicDirectiveArgumentEnd)
            .expect("missing outer argument delimiter");
        assert_eq!(error.loc.as_ref().unwrap().span.start as usize, boundary);
        assert_eq!(root.children.len(), 2, "{source}");
        let TemplateChildNode::Element(element) = &root.children[0] else {
            panic!("expected recovered component");
        };
        assert_eq!(element.props.len(), 2, "{source}");
        let PropNode::Directive(directive) = &element.props[0] else {
            panic!("expected recovered directive");
        };
        let Some(ExpressionNode::Simple(arg)) = &directive.arg else {
            panic!("expected recovered argument");
        };
        assert_eq!(arg.content, argument);
        let Some(ExpressionNode::Simple(value)) = &directive.exp else {
            panic!("expected recovered value");
        };
        assert_eq!(value.content, "value");
    }
}

#[test]
fn incomplete_dynamic_arguments_at_eof_keep_authored_content() {
    for argument in [
        "",
        "state.names[",
        "state.names[state.index]",
        "state.keys['broken",
        "`broken${state.key",
    ] {
        let source = cstr!("<Child :[{argument}");
        let allocator = Allocator::new();
        let (root, errors) = parse(&allocator, &source);
        assert!(errors.iter().any(|error| error.code == ErrorCode::EofInTag));
        assert!(
            errors
                .iter()
                .any(|error| error.code == ErrorCode::MissingDynamicDirectiveArgumentEnd)
        );
        assert_eq!(root.children.len(), 1);
        let TemplateChildNode::Element(element) = &root.children[0] else {
            panic!("expected recovered component");
        };
        let PropNode::Directive(directive) = &element.props[0] else {
            panic!("expected recovered directive");
        };
        if !argument.is_empty() {
            let Some(ExpressionNode::Simple(arg)) = &directive.arg else {
                panic!("expected recovered argument");
            };
            assert_eq!(arg.content, argument);
            assert!(!arg.is_static);
        }
    }
}
