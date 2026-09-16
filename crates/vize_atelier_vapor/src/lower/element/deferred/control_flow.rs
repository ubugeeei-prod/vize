//! Parent assignment for deferred structural children.

use super::{
    BlockIRNode, ElementNode, ElementType, OperationNode, TemplateChildNode, TransformContext,
};
use crate::ir::{ForIRNode, IfIRNode, NegativeBranch};
use crate::lower::control::{
    transform_for_node_deferred_parent, transform_for_node_into_parent,
    transform_if_node_deferred_parent, transform_if_node_into_parent,
};
use vize_carton::ensure_sufficient_stack;

fn transform_control_flow_children_into_parent<'a>(
    ctx: &mut TransformContext<'a>,
    children: &[TemplateChildNode<'a>],
    parent_id: usize,
    block: &mut BlockIRNode<'a>,
) {
    for child in vize_atelier_core::walk_probe::vapor_children(children) {
        match child {
            TemplateChildNode::If(if_node) => {
                transform_if_node_into_parent(ctx, if_node, block, parent_id);
            }
            TemplateChildNode::For(for_node) => {
                transform_for_node_into_parent(ctx, for_node, block, parent_id);
            }
            TemplateChildNode::Element(template) if template.tag_type == ElementType::Template => {
                ensure_sufficient_stack(|| {
                    transform_control_flow_children_into_parent(
                        ctx,
                        &template.children,
                        parent_id,
                        block,
                    );
                });
            }
            _ => {}
        }
    }
}

pub(super) fn transform_existing_element_control_flow_children<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    element_id: usize,
    block: &mut BlockIRNode<'a>,
) {
    transform_control_flow_children_into_parent(ctx, &el.children, element_id, block);
}

pub(super) fn transform_deferred_parent_control_flow_children<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    block: &mut BlockIRNode<'a>,
) {
    for child in vize_atelier_core::walk_probe::vapor_children(&el.children) {
        match child {
            TemplateChildNode::If(if_node) => {
                transform_if_node_deferred_parent(ctx, if_node, block);
            }
            TemplateChildNode::For(for_node) => {
                transform_for_node_deferred_parent(ctx, for_node, block);
            }
            TemplateChildNode::Element(template) if template.tag_type == ElementType::Template => {
                ensure_sufficient_stack(|| {
                    transform_deferred_parent_control_flow_children(ctx, template, block);
                });
            }
            _ => {}
        }
    }
}

pub(super) fn append_deferred_control_flow_children<'a>(
    block: &mut BlockIRNode<'a>,
    deferred_children: BlockIRNode<'a>,
    parent_id: usize,
) {
    for mut operation in deferred_children.operation {
        set_direct_control_flow_parent(&mut operation, parent_id);
        block.operation.push(operation);
    }
    for effect in deferred_children.effect {
        block.effect.push(effect);
    }
}

fn set_direct_control_flow_parent(operation: &mut OperationNode<'_>, parent_id: usize) {
    match operation {
        OperationNode::If(if_node) => set_if_parent(if_node, parent_id),
        OperationNode::For(for_node) => set_for_parent(for_node, parent_id),
        _ => {}
    }
}

fn set_if_parent(if_node: &mut IfIRNode<'_>, parent_id: usize) {
    if_node.parent = Some(parent_id);
    if let Some(NegativeBranch::If(nested_if)) = if_node.negative.as_mut() {
        set_if_parent(nested_if, parent_id);
    }
}

fn set_for_parent(for_node: &mut ForIRNode<'_>, parent_id: usize) {
    for_node.parent = Some(parent_id);
}
