//! v-for transform for Vapor mode.
//!
//! Transforms v-for directive into ForIRNode.

use vize_carton::{Box, Bump, String, ToCompactString};

use crate::ir::{BlockIRNode, ForIRNode, OperationNode};
use vize_atelier_core::{
    DirectiveNode, ElementNode, ElementType, ExpressionNode, ForNode, SimpleExpressionNode,
    SourceLocation,
};

/// Transform v-for directive to IR
pub fn transform_v_for<'a>(
    allocator: &'a Bump,
    dir: &DirectiveNode<'a>,
    el: &ElementNode<'a>,
    render_block: BlockIRNode<'a>,
    id: usize,
) -> OperationNode<'a> {
    let source = if let Some(ref exp) = dir.exp {
        extract_expression(allocator, exp)
    } else {
        Box::new_in(
            SimpleExpressionNode::new("[]", false, SourceLocation::STUB),
            allocator,
        )
    };

    let for_node = ForIRNode {
        id,
        source,
        value: None,
        key: None,
        index: None,
        key_prop: None,
        render: render_block,
        once: false,
        component: el.tag_type == ElementType::Component,
        only_child: false,
        parent: None,
        anchor: None,
    };

    OperationNode::For(Box::new_in(for_node, allocator))
}

/// Transform ForNode (from compiler-core) to ForIRNode
pub fn transform_for_node<'a>(
    allocator: &'a Bump,
    for_node: &ForNode<'a>,
    render_block: BlockIRNode<'a>,
    id: usize,
) -> OperationNode<'a> {
    let source = extract_expression(allocator, &for_node.source);

    let value = for_node
        .value_alias
        .as_ref()
        .map(|v| extract_expression(allocator, v));
    let key = for_node
        .key_alias
        .as_ref()
        .map(|k| extract_expression(allocator, k));
    let index = for_node
        .object_index_alias
        .as_ref()
        .map(|i| extract_expression(allocator, i));

    let for_ir = ForIRNode {
        id,
        source,
        value,
        key,
        index,
        key_prop: None,
        render: render_block,
        once: false,
        component: false,
        only_child: for_node.children.len() == 1,
        parent: None,
        anchor: None,
    };

    OperationNode::For(Box::new_in(for_ir, allocator))
}

/// Extract expression from ExpressionNode
fn extract_expression<'a>(
    allocator: &'a Bump,
    exp: &ExpressionNode<'a>,
) -> Box<'a, SimpleExpressionNode<'a>> {
    match exp {
        ExpressionNode::Simple(simple) => {
            let node = SimpleExpressionNode::new(
                simple.content.clone(),
                simple.is_static,
                simple.loc.clone(),
            );
            Box::new_in(node, allocator)
        }
        ExpressionNode::Compound(compound) => {
            let node =
                SimpleExpressionNode::new(compound.loc.source.clone(), false, compound.loc.clone());
            Box::new_in(node, allocator)
        }
    }
}

/// Parse v-for expression alias pattern
/// Returns (value, key, index) aliases
pub fn parse_for_alias(content: &str) -> (Option<String>, Option<String>, Option<String>) {
    let content = content.trim();

    // Handle "(item, index)" or "(item, key, index)" patterns
    if content.starts_with('(') && content.ends_with(')') {
        let inner = &content[1..content.len() - 1];
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();

        let value = parts
            .first()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_compact_string());
        let key = parts
            .get(1)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_compact_string());
        let index = parts
            .get(2)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_compact_string());

        return (value, key, index);
    }

    // Single value pattern
    if !content.is_empty() {
        return (Some(content.to_compact_string()), None, None);
    }

    (None, None, None)
}

#[cfg(test)]
mod tests {
    use super::parse_for_alias;
    use vize_carton::ToCompactString;

    #[test]
    fn test_parse_for_alias_simple() {
        let (value, key, index) = parse_for_alias("item");
        assert_eq!(value, Some("item".to_compact_string()));
        assert_eq!(key, None);
        assert_eq!(index, None);
    }

    #[test]
    fn test_parse_for_alias_with_index() {
        let (value, key, index) = parse_for_alias("(item, index)");
        assert_eq!(value, Some("item".to_compact_string()));
        assert_eq!(key, Some("index".to_compact_string()));
        assert_eq!(index, None);
    }

    #[test]
    fn test_parse_for_alias_with_key_and_index() {
        let (value, key, index) = parse_for_alias("(value, key, index)");
        assert_eq!(value, Some("value".to_compact_string()));
        assert_eq!(key, Some("key".to_compact_string()));
        assert_eq!(index, Some("index".to_compact_string()));
    }
}
