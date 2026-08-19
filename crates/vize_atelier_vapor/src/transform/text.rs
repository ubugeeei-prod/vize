//! Text and interpolation transformation.
//!
//! Handles `TextNode`, `InterpolationNode`, and mixed text/interpolation children.

use vize_carton::{Box, Vec, ensure_sufficient_stack};

use crate::ir::{BlockIRNode, ChildRefIRNode, OperationNode, SetTextIRNode};
use vize_atelier_core::{
    ElementType, ExpressionNode, InterpolationNode, SimpleExpressionNode, SourceLocation,
    TemplateChildNode, TextNode,
};

use super::context::TransformContext;

/// Transform text node
pub(crate) fn transform_text<'a>(
    ctx: &mut TransformContext<'a>,
    text: &TextNode,
    block: &mut BlockIRNode<'a>,
) {
    let element_id = ctx.next_id();
    let template: vize_carton::String = text.content.into();
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
            let mut v = Vec::new_in(&ctx.allocator);
            let exp = SimpleExpressionNode::from_node(simple);
            v.push(Box::new_in(exp, &ctx.allocator));
            v
        }
        _ => Vec::new_in(&ctx.allocator),
    };

    let set_text = SetTextIRNode {
        element: element_id,
        values,
    };

    ctx.push_dynamic_operation(block, OperationNode::SetText(set_text));

    block.returns.push(element_id);
}

/// Transform text children (combined text and interpolations)
pub(crate) fn transform_text_children<'a>(
    ctx: &mut TransformContext<'a>,
    children: &[TemplateChildNode<'a>],
    parent_element_id: usize,
    block: &mut BlockIRNode<'a>,
) {
    let mut values = Vec::new_in(&ctx.allocator);
    let mut has_interpolation = false;
    let mut run_index = None;
    let mut rendered_index = 0usize;
    collect_text_runs(
        ctx,
        children,
        parent_element_id,
        block,
        &mut values,
        &mut has_interpolation,
        &mut run_index,
        &mut rendered_index,
    );
    flush_text_run(
        ctx,
        parent_element_id,
        block,
        &mut values,
        &mut has_interpolation,
        &mut run_index,
    );
}

#[allow(clippy::too_many_arguments)]
fn collect_text_runs<'a>(
    ctx: &mut TransformContext<'a>,
    children: &[TemplateChildNode<'a>],
    parent_element_id: usize,
    block: &mut BlockIRNode<'a>,
    values: &mut Vec<'a, Box<'a, SimpleExpressionNode<'a>>>,
    has_interpolation: &mut bool,
    run_index: &mut Option<usize>,
    rendered_index: &mut usize,
) {
    for child in vize_atelier_core::walk_probe::vapor_children(children) {
        match child {
            TemplateChildNode::Text(text) => {
                begin_text_run(run_index, rendered_index);
                let exp = SimpleExpressionNode::new(text.content, true, SourceLocation::STUB);
                values.push(Box::new_in(exp, &ctx.allocator));
            }
            TemplateChildNode::Interpolation(interp) => {
                begin_text_run(run_index, rendered_index);
                // Dynamic interpolation
                if let ExpressionNode::Simple(simple) = &interp.content {
                    let exp = SimpleExpressionNode::from_node(simple);
                    values.push(Box::new_in(exp, &ctx.allocator));
                    *has_interpolation = true;
                }
            }
            TemplateChildNode::Element(template) if template.tag_type == ElementType::Template => {
                ensure_sufficient_stack(|| {
                    collect_text_runs(
                        ctx,
                        &template.children,
                        parent_element_id,
                        block,
                        values,
                        has_interpolation,
                        run_index,
                        rendered_index,
                    );
                });
            }
            TemplateChildNode::Element(element) if element.tag_type == ElementType::Element => {
                flush_text_run(
                    ctx,
                    parent_element_id,
                    block,
                    values,
                    has_interpolation,
                    run_index,
                );
                *rendered_index += 1;
            }
            _ => {}
        }
    }
}

fn begin_text_run(run_index: &mut Option<usize>, rendered_index: &mut usize) {
    if run_index.is_none() {
        *run_index = Some(*rendered_index);
        *rendered_index += 1;
    }
}

fn flush_text_run<'a>(
    ctx: &mut TransformContext<'a>,
    parent_element_id: usize,
    block: &mut BlockIRNode<'a>,
    values: &mut Vec<'a, Box<'a, SimpleExpressionNode<'a>>>,
    has_interpolation: &mut bool,
    run_index: &mut Option<usize>,
) {
    let Some(offset) = run_index.take() else {
        return;
    };
    if *has_interpolation {
        let text_id = if offset == 0 {
            parent_element_id
        } else {
            let text_id = ctx.next_id();
            block
                .operation
                .push(OperationNode::ChildRef(ChildRefIRNode {
                    child_id: text_id,
                    parent_id: parent_element_id,
                    offset,
                }));
            ctx.standalone_text_elements.insert(text_id);
            text_id
        };
        let run_values = std::mem::replace(values, Vec::new_in(&ctx.allocator));
        ctx.push_dynamic_operation(
            block,
            OperationNode::SetText(SetTextIRNode {
                element: text_id,
                values: run_values,
            }),
        );
    } else {
        values.clear();
    }
    *has_interpolation = false;
}
