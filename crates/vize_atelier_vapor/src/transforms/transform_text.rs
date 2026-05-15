//! Text transform for Vapor mode.
//!
//! Transforms text and interpolation nodes into SetTextIRNode.

use vize_carton::{cstr, Box, Bump, String, Vec};

use crate::ir::{OperationNode, SetTextIRNode};
use vize_atelier_core::{ExpressionNode, InterpolationNode, SimpleExpressionNode, TextNode};

/// Transform interpolation to SetTextIRNode
pub fn transform_interpolation<'a>(
    allocator: &'a Bump,
    interp: &InterpolationNode<'a>,
    element_id: usize,
) -> (OperationNode<'a>, bool) {
    let values = extract_text_values(allocator, &interp.content);

    let set_text = SetTextIRNode {
        element: element_id,
        values,
    };

    // Interpolations are always reactive
    let is_reactive = true;

    (OperationNode::SetText(set_text), is_reactive)
}

/// Transform text node (static text doesn't need SetTextIRNode)
pub fn transform_text<'a>(
    _allocator: &'a Bump,
    _text: &TextNode,
    _element_id: usize,
) -> Option<OperationNode<'a>> {
    // Static text is included in the template string
    // Only return operation if we need dynamic text handling
    None
}

/// Extract text values from expression
fn extract_text_values<'a>(
    allocator: &'a Bump,
    exp: &ExpressionNode<'a>,
) -> Vec<'a, Box<'a, SimpleExpressionNode<'a>>> {
    let mut values = Vec::new_in(allocator);

    match exp {
        ExpressionNode::Simple(simple) => {
            let node = SimpleExpressionNode::new(
                simple.content.clone(),
                simple.is_static,
                simple.loc.clone(),
            );
            values.push(Box::new_in(node, allocator));
        }
        ExpressionNode::Compound(compound) => {
            // For compound expressions, extract as a single value
            let node =
                SimpleExpressionNode::new(compound.loc.source.clone(), false, compound.loc.clone());
            values.push(Box::new_in(node, allocator));
        }
    }

    values
}

/// Merge consecutive text/interpolation nodes
pub fn should_merge_text_nodes(children: &[vize_atelier_core::TemplateChildNode<'_>]) -> bool {
    let mut consecutive_count = 0;
    for child in children {
        match child {
            vize_atelier_core::TemplateChildNode::Text(_)
            | vize_atelier_core::TemplateChildNode::Interpolation(_) => {
                consecutive_count += 1;
                if consecutive_count >= 2 {
                    return true;
                }
            }
            _ => {
                consecutive_count = 0;
            }
        }
    }
    false
}

/// Generate text call expression
pub fn generate_text_expression(parts: &[(bool, String)]) -> String {
    if parts.is_empty() {
        return String::from("\"\"");
    }

    if parts.len() == 1 {
        let (is_static, content) = &parts[0];
        if *is_static {
            return cstr!("\"{}\"", escape_text(content));
        } else {
            return cstr!("_toDisplayString({content})");
        }
    }

    // Multiple parts - concatenate with +
    let exprs: std::vec::Vec<vize_carton::CompactString> = parts
        .iter()
        .map(|(is_static, content)| {
            if *is_static {
                cstr!("\"{}\"", escape_text(content))
            } else {
                cstr!("_toDisplayString({content})")
            }
        })
        .collect();

    exprs.join(" + ").into()
}

/// Escape text for JavaScript string
fn escape_text(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .into()
}

#[cfg(test)]
mod tests {
    use super::generate_text_expression;
    use vize_carton::String;

    #[test]
    fn test_generate_text_expression_static() {
        let parts = vec![(true, String::new("hello"))];
        let result = generate_text_expression(&parts);
        assert_eq!(result, "\"hello\"");
    }

    #[test]
    fn test_generate_text_expression_dynamic() {
        let parts = vec![(false, String::new("msg"))];
        let result = generate_text_expression(&parts);
        assert_eq!(result, "_toDisplayString(msg)");
    }

    #[test]
    fn test_generate_text_expression_mixed() {
        let parts = vec![
            (true, String::new("Hello, ")),
            (false, String::new("name")),
            (true, String::new("!")),
        ];
        let result = generate_text_expression(&parts);
        insta::assert_snapshot!(result.as_str());
    }
}
