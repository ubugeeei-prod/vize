#![expect(clippy::disallowed_macros, reason = "insta and fixtures use format!")]

use super::{transform, transform_with_template_syntax_quirks};
use crate::codegen::generate;
use crate::options::{CodegenOptions, TransformOptions};
use crate::parser::parse;
use vize_l0::{Allocator, Span};

#[test]
fn test_transform_simple_element() {
    assert_transform!("<div>hello</div>" => helpers: [CreateElementVNode]);
}

#[test]
fn test_transform_interpolation() {
    assert_transform!("{{ msg }}" => helpers: [ToDisplayString]);
}

#[test]
fn test_transform_component() {
    assert_transform!("<MyComponent></MyComponent>" => components: ["MyComponent"]);
    assert_transform!("<MyComponent></MyComponent>" => helpers: [ResolveComponent]);
}

#[test]
fn test_transform_pascal_case_dynamic_component() {
    let allocator = Allocator::new();
    let (mut root, errors) = parse(&allocator, r#"<Component :is="current" />"#);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    transform(&allocator, &mut root, TransformOptions::default(), None);

    assert!(
        !root.components.contains(&"Component"),
        "Dynamic component special tag should not be tracked as a resolved component"
    );
    assert!(
        !root
            .helpers
            .iter()
            .any(|helper| matches!(helper, crate::RuntimeHelper::ResolveComponent)),
        "Dynamic component special tag should not request resolveComponent"
    );
}

#[test]
fn test_transform_v_if() {
    assert_transform!("<div v-if=\"show\">hello</div>" => helpers: [OpenBlock, CreateBlock, Fragment, CreateComment]);
}

#[test]
fn test_transform_v_for() {
    assert_transform!("<div v-for=\"item in items\">{{ item }}</div>" => helpers: [RenderList, OpenBlock, CreateBlock, Fragment]);
}

#[test]
fn v_for_alias_rewrites_keep_cooked_names_and_nested_scope() {
    let allocator = Allocator::new();
    let source = r#"<div v-for="outer in rows"><p v-for="{ \u006cabel = fallback, [key]: tag, nested: { name = pick(name, other) } } in outer.items">{{ label }}{{ tag }}{{ name }}{{ outside }}</p></div>"#;
    let (mut root, errors) = parse(&allocator, source);
    assert!(errors.is_empty(), "Parse errors: {errors:?}");

    let errors = transform(
        &allocator,
        &mut root,
        TransformOptions {
            prefix_identifiers: true,
            ..Default::default()
        },
        None,
    );
    assert!(errors.is_empty(), "Transform errors: {errors:?}");

    let crate::TemplateChildNode::For(outer) = &root.children[0] else {
        panic!("expected outer ForNode")
    };
    let Some(crate::ExpressionNode::Simple(outer_alias)) = &outer.value_alias else {
        panic!("expected outer value alias")
    };
    assert_eq!(outer_alias.content, "outer");
    assert!(outer_alias.js_ast.is_none());
    assert!(outer_alias.identifiers.is_none());
    let crate::TemplateChildNode::Element(outer_element) = &outer.children[0] else {
        panic!("expected outer element")
    };
    let crate::TemplateChildNode::For(inner) = &outer_element.children[0] else {
        panic!("expected inner ForNode")
    };
    let crate::ExpressionNode::Simple(source) = &inner.source else {
        panic!("expected inner source")
    };
    assert_eq!(source.content, "outer.items");
    let Some(crate::ExpressionNode::Simple(alias)) = &inner.value_alias else {
        panic!("expected value alias")
    };
    assert_eq!(
        alias.content,
        r#"{ \u006cabel = _ctx.fallback, [_ctx.key]: tag, nested: { name = _ctx.pick(name, _ctx.other) } }"#
    );
    assert_eq!(
        alias.retained_parameter_names().unwrap(),
        ["label", "tag", "name"]
    );

    let crate::TemplateChildNode::Element(inner_element) = &inner.children[0] else {
        panic!("expected inner element")
    };
    let expressions: std::vec::Vec<_> = inner_element
        .children
        .iter()
        .map(|child| match child {
            crate::TemplateChildNode::Interpolation(interpolation) => {
                match &interpolation.content {
                    crate::ExpressionNode::Simple(expression) => expression.content,
                    crate::ExpressionNode::Compound(_) => panic!("expected simple interpolation"),
                }
            }
            _ => panic!("expected interpolation"),
        })
        .collect();
    assert_eq!(expressions, ["label", "tag", "name", "_ctx.outside"]);
}

#[test]
fn typed_v_for_alias_rewrites_runtime_defaults_without_prefixing_types() {
    let allocator = Allocator::new();
    let source = r#"<p v-for="({ label = fallback }: Row) in items">{{ label }}{{ outside }}</p>"#;
    let (mut root, errors) = parse(&allocator, source);
    assert!(errors.is_empty(), "Parse errors: {errors:?}");

    let errors = transform(
        &allocator,
        &mut root,
        TransformOptions {
            prefix_identifiers: true,
            is_ts: true,
            ..Default::default()
        },
        None,
    );
    assert!(errors.is_empty(), "Transform errors: {errors:?}");

    let crate::TemplateChildNode::For(for_node) = &root.children[0] else {
        panic!("expected ForNode")
    };
    let Some(crate::ExpressionNode::Simple(alias)) = &for_node.value_alias else {
        panic!("expected value alias")
    };
    assert_eq!(alias.content, "{ label = _ctx.fallback }: Row");
    assert_eq!(alias.retained_parameter_names().unwrap(), ["label"]);
    let crate::ExpressionNode::Simple(iterable) = &for_node.source else {
        panic!("expected simple iterable")
    };
    assert_eq!(iterable.content, "_ctx.items");

    let crate::TemplateChildNode::Element(element) = &for_node.children[0] else {
        panic!("expected loop element")
    };
    let expressions: std::vec::Vec<_> = element
        .children
        .iter()
        .map(|child| match child {
            crate::TemplateChildNode::Interpolation(interpolation) => {
                match &interpolation.content {
                    crate::ExpressionNode::Simple(expression) => expression.content,
                    crate::ExpressionNode::Compound(_) => panic!("expected simple interpolation"),
                }
            }
            _ => panic!("expected interpolation"),
        })
        .collect();
    assert_eq!(expressions, ["label", "_ctx.outside"]);
}

#[test]
fn v_for_alias_parse_errors_use_authored_expression_span() {
    let allocator = Allocator::new();
    let expression = "({ label = fallback }: Row) in items";
    let source = format!(r#"<p v-for="{expression}"></p>"#);
    let (mut root, errors) = parse(&allocator, &source);
    assert!(errors.is_empty(), "Parse errors: {errors:?}");

    let errors = transform(
        &allocator,
        &mut root,
        TransformOptions {
            prefix_identifiers: true,
            ..Default::default()
        },
        None,
    );
    assert_eq!(errors.len(), 1, "Transform errors: {errors:?}");
    let error = &errors[0];
    assert_eq!(error.code, crate::errors::ErrorCode::InvalidExpression);

    let start = source.find(expression).unwrap() as u32;
    let expected = Span::new(start, start + expression.len() as u32);
    assert_eq!(error.loc.as_ref().map(|loc| loc.span), Some(expected));
    assert_eq!(error.loc.as_ref().unwrap().span.slice(&source), expression);
}

#[test]
fn test_transform_v_for_rejects_unmatched_edge_parens_by_default() {
    let allocator = Allocator::new();
    let (mut root, errors) = parse(&allocator, r#"<div v-for="item) in items"></div>"#);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    transform(&allocator, &mut root, TransformOptions::default(), None);

    assert!(
        !matches!(&root.children[0], crate::TemplateChildNode::For(_)),
        "strict parser mode should not accept unmatched v-for alias parens"
    );
}

#[test]
fn test_transform_v_for_template_syntax_quirks_accepts_unmatched_edge_parens() {
    let allocator = Allocator::new();
    let (mut root, errors) = parse(&allocator, r#"<div v-for="item) in items"></div>"#);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    transform_with_template_syntax_quirks(&allocator, &mut root, TransformOptions::default(), None);

    match &root.children[0] {
        crate::TemplateChildNode::For(for_node) => match &for_node.value_alias {
            Some(crate::ExpressionNode::Simple(value)) => {
                assert_eq!(value.content, "item");
            }
            _ => panic!("expected value alias"),
        },
        other => panic!("expected ForNode, got {:?}", std::mem::discriminant(other)),
    }
}

#[test]
fn test_v_if_creates_if_node() {
    let allocator = Allocator::new();
    let (mut root, errors) = parse(&allocator, r#"<div v-if="show">visible</div>"#);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    transform(&allocator, &mut root, TransformOptions::default(), None);

    // After transform, root should have 1 child: an IfNode
    assert_eq!(
        root.children.len(),
        1,
        "Should have 1 child after transform"
    );

    match &root.children[0] {
        crate::TemplateChildNode::If(if_node) => {
            assert_eq!(if_node.branches.len(), 1, "Should have 1 branch");
            // First branch should have condition "show"
            let branch = &if_node.branches[0];
            assert!(branch.condition.is_some(), "Branch should have condition");
        }
        other => panic!("Expected IfNode, got {:?}", std::mem::discriminant(other)),
    }
}

#[test]
fn test_v_if_else_creates_branches() {
    let allocator = Allocator::new();
    let (mut root, errors) = parse(
        &allocator,
        r#"<div v-if="show">yes</div><div v-else>no</div>"#,
    );
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    transform(&allocator, &mut root, TransformOptions::default(), None);

    // After transform, should have 1 IfNode with 2 branches
    assert_eq!(
        root.children.len(),
        1,
        "Should have 1 child (IfNode) after transform, got {}",
        root.children.len()
    );

    match &root.children[0] {
        crate::TemplateChildNode::If(if_node) => {
            assert_eq!(
                if_node.branches.len(),
                2,
                "Should have 2 branches (if + else)"
            );
            // First branch has condition, second doesn't (v-else)
            assert!(
                if_node.branches[0].condition.is_some(),
                "First branch should have condition"
            );
            assert!(
                if_node.branches[1].condition.is_none(),
                "Second branch (else) should not have condition"
            );
        }
        other => panic!("Expected IfNode, got {:?}", std::mem::discriminant(other)),
    }
}

#[test]
fn test_v_for_creates_for_node() {
    let allocator = Allocator::new();
    let (mut root, errors) = parse(&allocator, r#"<div v-for="item in items">{{ item }}</div>"#);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    transform(&allocator, &mut root, TransformOptions::default(), None);

    // After transform, root should have 1 child: a ForNode
    assert_eq!(
        root.children.len(),
        1,
        "Should have 1 child after transform"
    );

    match &root.children[0] {
        crate::TemplateChildNode::For(for_node) => {
            // Check source is "items"
            match &for_node.source {
                crate::ExpressionNode::Simple(exp) => {
                    assert_eq!(exp.content, "items", "Source should be 'items'");
                }
                _ => panic!("Expected Simple expression for source"),
            }
            // Check value alias is "item"
            assert!(for_node.value_alias.is_some(), "Should have value alias");
            match for_node.value_alias.as_ref().unwrap() {
                crate::ExpressionNode::Simple(exp) => {
                    assert_eq!(exp.content, "item", "Value alias should be 'item'");
                }
                _ => panic!("Expected Simple expression for value alias"),
            }
        }
        other => panic!("Expected ForNode, got {:?}", std::mem::discriminant(other)),
    }
}

#[test]
fn test_codegen_v_if() {
    let allocator = Allocator::new();
    let (mut root, _) = parse(&allocator, r#"<div v-if="show">visible</div>"#);
    transform(&allocator, &mut root, TransformOptions::default(), None);

    let result = generate(&root, CodegenOptions::default());
    insta::assert_snapshot!(result.code.as_str());
}
