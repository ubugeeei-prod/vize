//! Text and interpolation transformation.
//!
//! Handles `TextNode`, `InterpolationNode`, and mixed text/interpolation children.

use vize_carton::{Box, Vec};

use crate::ir::{BlockIRNode, IREffect, OperationNode, SetTextIRNode};
use vize_atelier_core::{
    ExpressionNode, InterpolationNode, SimpleExpressionNode, SourceLocation, TemplateChildNode,
    TextNode,
};

use super::context::TransformContext;

/// Transform text node
pub(crate) fn transform_text<'a>(
    ctx: &mut TransformContext<'a>,
    text: &TextNode,
    block: &mut BlockIRNode<'a>,
) {
    let element_id = ctx.next_id();
    let template: vize_carton::String = text.content.clone();
    ctx.add_template(element_id, template);
    block.returns.push(element_id);
}

/// Transform interpolation node (standalone, not inside element)
pub(crate) fn transform_interpolation<'a>(
    ctx: &mut TransformContext<'a>,
    interp: &InterpolationNode<'a>,
    block: &mut BlockIRNode<'a>,
) {
    let element_id = ctx.next_id();

    // Register a space placeholder template for standalone interpolations
    // (when not inside a parent element that already provides the template)
    ctx.add_template(element_id, vize_carton::String::from(" "));
    ctx.standalone_text_elements.insert(element_id);

    // Create SetText operation
    let values = match &interp.content {
        ExpressionNode::Simple(simple) => {
            let mut v = Vec::new_in(ctx.allocator);
            let exp = SimpleExpressionNode::new(
                simple.content.clone(),
                simple.is_static,
                simple.loc.clone(),
            );
            v.push(Box::new_in(exp, ctx.allocator));
            v
        }
        _ => Vec::new_in(ctx.allocator),
    };

    let set_text = SetTextIRNode {
        element: element_id,
        values,
    };

    // Add to effects (reactive)
    let mut effect_ops = Vec::new_in(ctx.allocator);
    effect_ops.push(OperationNode::SetText(set_text));

    block.effect.push(IREffect {
        operations: effect_ops,
    });

    block.returns.push(element_id);
}

/// Transform text children (combined text and interpolations)
pub(crate) fn transform_text_children<'a>(
    ctx: &mut TransformContext<'a>,
    children: &[TemplateChildNode<'a>],
    parent_element_id: usize,
    block: &mut BlockIRNode<'a>,
) {
    let mut values = Vec::new_in(ctx.allocator);

    // Collect all text parts and interpolations
    for child in children.iter() {
        match child {
            TemplateChildNode::Text(text) => {
                // Static text part
                let exp = SimpleExpressionNode::new(
                    text.content.clone(),
                    true, // is_static = true
                    SourceLocation::STUB,
                );
                values.push(Box::new_in(exp, ctx.allocator));
            }
            TemplateChildNode::Interpolation(interp) => {
                // Dynamic interpolation
                if let ExpressionNode::Simple(simple) = &interp.content {
                    let exp = SimpleExpressionNode::new(
                        simple.content.clone(),
                        simple.is_static,
                        simple.loc.clone(),
                    );
                    values.push(Box::new_in(exp, ctx.allocator));
                }
            }
            _ => {}
        }
    }

    if !values.is_empty() {
        let set_text = SetTextIRNode {
            element: parent_element_id,
            values,
        };

        let mut effect_ops = Vec::new_in(ctx.allocator);
        effect_ops.push(OperationNode::SetText(set_text));

        block.effect.push(IREffect {
            operations: effect_ops,
        });
    }
}
