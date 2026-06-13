#![allow(clippy::disallowed_macros)]

use super::{transform, transform_with_template_syntax_quirks};
use crate::codegen::generate;
use crate::options::{CodegenOptions, TransformOptions};
use crate::parser::parse;
use bumpalo::Bump;

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
    let allocator = Bump::new();
    let (mut root, errors) = parse(&allocator, r#"<Component :is="current" />"#);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    transform(&allocator, &mut root, TransformOptions::default(), None);

    assert!(
        !root
            .components
            .iter()
            .any(|component| component.as_str() == "Component"),
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
fn test_transform_v_for_rejects_unmatched_edge_parens_by_default() {
    let allocator = Bump::new();
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
    let allocator = Bump::new();
    let (mut root, errors) = parse(&allocator, r#"<div v-for="item) in items"></div>"#);
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    transform_with_template_syntax_quirks(&allocator, &mut root, TransformOptions::default(), None);

    match &root.children[0] {
        crate::TemplateChildNode::For(for_node) => match &for_node.value_alias {
            Some(crate::ExpressionNode::Simple(value)) => {
                assert_eq!(value.content.as_str(), "item");
            }
            _ => panic!("expected value alias"),
        },
        other => panic!("expected ForNode, got {:?}", std::mem::discriminant(other)),
    }
}

#[test]
fn test_v_if_creates_if_node() {
    let allocator = Bump::new();
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
    let allocator = Bump::new();
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
    let allocator = Bump::new();
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
                    assert_eq!(exp.content.as_str(), "items", "Source should be 'items'");
                }
                _ => panic!("Expected Simple expression for source"),
            }
            // Check value alias is "item"
            assert!(for_node.value_alias.is_some(), "Should have value alias");
            match for_node.value_alias.as_ref().unwrap() {
                crate::ExpressionNode::Simple(exp) => {
                    assert_eq!(exp.content.as_str(), "item", "Value alias should be 'item'");
                }
                _ => panic!("Expected Simple expression for value alias"),
            }
        }
        other => panic!("Expected ForNode, got {:?}", std::mem::discriminant(other)),
    }
}

#[test]
fn test_codegen_v_if() {
    let allocator = Bump::new();
    let (mut root, _) = parse(&allocator, r#"<div v-if="show">visible</div>"#);
    transform(&allocator, &mut root, TransformOptions::default(), None);

    let result = generate(&root, CodegenOptions::default());
    insta::assert_snapshot!(result.code.as_str());
}
