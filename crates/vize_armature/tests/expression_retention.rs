use vize_armature::{Allocator, ExpressionNode, PropNode, TemplateChildNode, parse};

#[test]
fn directive_retains_expression_with_trailing_block_comment() {
    let allocator = Allocator::new();
    let (root, errors) = parse(
        &allocator,
        r#"<circle v-if="i % 3 === 0 /* perf optimization */"></circle>"#,
    );

    assert!(errors.is_empty());
    let TemplateChildNode::Element(el) = &root.children[0] else {
        panic!("expected element node");
    };
    let PropNode::Directive(dir) = &el.props[0] else {
        panic!("expected directive");
    };
    assert_eq!(dir.name, "if");
    let Some(ExpressionNode::Simple(exp)) = &dir.exp else {
        panic!("expected simple expression");
    };
    assert_eq!(exp.content, "i % 3 === 0 /* perf optimization */");
    assert!(exp.js_ast.is_some());
}
