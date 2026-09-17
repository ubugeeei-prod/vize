//! Deferred child ID allocation for dynamic and control-flow descendants.

use crate::ir::InsertNodeIRNode;
use vize_carton::ensure_sufficient_stack;

use super::component::transform_component;
use super::template::{
    generate_element_template, is_static_element, is_template_backed_element,
    transform_template_ref,
};
use super::{
    BlockIRNode, ChildRefIRNode, ElementNode, ElementType, NextRefIRNode, OperationNode, PropNode,
    SlotOutletIRNode, TemplateChildNode, TransformContext, get_slot_outlet_name,
    get_slot_outlet_props, transform_children, transform_directive, transform_for_node_into_parent,
    transform_if_node_into_parent, transform_text_children,
};

/// Transform an element that has control flow children (`v-if`/`v-for`).
///
/// Structural and component children share the same authored insertion anchors.
pub(super) fn transform_element_with_control_flow_children<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    block: &mut BlockIRNode<'a>,
) {
    transform_element_with_dynamic_children(ctx, el, block);
}

/// Transform an element that has dynamic element children.
///
/// Child IDs are allocated before the parent ID, and `ChildRef`/`NextRef`
/// operations are used instead of separate templates for each child.
pub(super) fn transform_element_with_dynamic_children<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    block: &mut BlockIRNode<'a>,
) {
    let dynamic_child_count = count_dynamic_element_children(&el.children);
    let child_ids: std::vec::Vec<usize> = (0..dynamic_child_count).map(|_| ctx.next_id()).collect();

    // Now allocate parent ID (will be higher than all child IDs)
    let parent_id = ctx.next_id();

    // Generate template (includes all children inline)
    let template = generate_element_template(el, ctx.scope_id.as_deref());

    // Process parent props
    for prop in el.props.iter() {
        match prop {
            PropNode::Directive(dir) => {
                transform_directive(ctx, dir, parent_id, el, block);
            }
            PropNode::Attribute(_attr) => {}
        }
    }

    transform_template_ref(ctx, el, parent_id, block);
    transform_text_children(ctx, &el.children, parent_id, block);

    transform_dynamic_children_with_ids(ctx, el, parent_id, block, &child_ids);

    // Register template for parent
    ctx.add_template(parent_id, template);

    block.returns.push(parent_id);
}

fn count_dynamic_element_children(children: &[TemplateChildNode<'_>]) -> usize {
    children
        .iter()
        .map(|child| match child {
            TemplateChildNode::Element(child_el) if child_el.tag_type == ElementType::Template => {
                ensure_sufficient_stack(|| count_dynamic_element_children(&child_el.children))
            }
            TemplateChildNode::Element(child_el) if !is_static_element(child_el) => 1,
            _ => 0,
        })
        .sum()
}

fn transform_dynamic_children_with_ids<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    parent_id: usize,
    block: &mut BlockIRNode<'a>,
    child_ids: &[usize],
) {
    // (child id, absolute rendered index within the parent)
    let mut prev_template_backed_child: Option<(usize, usize)> = None;
    let mut child_id_index = 0usize;
    let mut rendered_index = 0usize;
    let mut in_text_run = false;
    transform_dynamic_children_in_slice(
        ctx,
        &el.children,
        parent_id,
        block,
        child_ids,
        &mut child_id_index,
        &mut rendered_index,
        &mut in_text_run,
        &mut prev_template_backed_child,
    );
    debug_assert_eq!(child_id_index, child_ids.len());
}

#[allow(clippy::too_many_arguments)]
fn transform_dynamic_children_in_slice<'a>(
    ctx: &mut TransformContext<'a>,
    children: &[TemplateChildNode<'a>],
    parent_id: usize,
    block: &mut BlockIRNode<'a>,
    child_ids: &[usize],
    child_id_index: &mut usize,
    rendered_index: &mut usize,
    in_text_run: &mut bool,
    prev_template_backed_child: &mut Option<(usize, usize)>,
) {
    for child in vize_atelier_core::walk_probe::vapor_children(children) {
        let TemplateChildNode::Element(child_el) = child else {
            match child {
                TemplateChildNode::If(node) => {
                    let anchor = insertion_anchor(ctx, block, parent_id, *rendered_index);
                    transform_if_node_into_parent(ctx, node, block, parent_id, anchor);
                    *rendered_index += 1;
                    *in_text_run = false;
                }
                TemplateChildNode::For(node) => {
                    let anchor = insertion_anchor(ctx, block, parent_id, *rendered_index);
                    transform_for_node_into_parent(ctx, node, block, parent_id, anchor);
                    *rendered_index += 1;
                    *in_text_run = false;
                }
                _ => {}
            }
            if matches!(
                child,
                TemplateChildNode::Text(_) | TemplateChildNode::Interpolation(_)
            ) && !*in_text_run
            {
                *rendered_index += 1;
                *in_text_run = true;
            }
            continue;
        };

        if child_el.tag_type == ElementType::Template {
            ensure_sufficient_stack(|| {
                transform_dynamic_children_in_slice(
                    ctx,
                    &child_el.children,
                    parent_id,
                    block,
                    child_ids,
                    child_id_index,
                    rendered_index,
                    in_text_run,
                    prev_template_backed_child,
                );
            });
            continue;
        }

        if !is_static_element(child_el) {
            let child_id = child_ids[*child_id_index];
            *child_id_index += 1;

            if is_template_backed_element(child_el) {
                let index = *rendered_index;
                if let Some((prev_child_id, prev_index)) = *prev_template_backed_child {
                    block.operation.push(OperationNode::NextRef(NextRefIRNode {
                        child_id,
                        prev_id: prev_child_id,
                        offset: index.saturating_sub(prev_index),
                    }));
                } else {
                    block
                        .operation
                        .push(OperationNode::ChildRef(ChildRefIRNode {
                            child_id,
                            parent_id,
                            offset: index,
                        }));
                }

                *prev_template_backed_child = Some((child_id, index));
                ensure_sufficient_stack(|| {
                    transform_existing_element(ctx, child_el, child_id, block);
                });
            } else if child_el.tag_type == ElementType::Slot {
                let anchor = insertion_anchor(ctx, block, parent_id, *rendered_index);
                transform_slot_outlet_child(ctx, child_el, child_id, parent_id, anchor, block);
            } else {
                let anchor = insertion_anchor(ctx, block, parent_id, *rendered_index);
                transform_component(
                    ctx,
                    child_el,
                    block,
                    Some(child_id),
                    Some(parent_id),
                    Some(anchor),
                    false,
                );
            }
        }

        *rendered_index += 1;
        *in_text_run = false;
    }
}

fn insertion_anchor<'a>(
    ctx: &mut TransformContext<'a>,
    block: &mut BlockIRNode<'a>,
    parent_id: usize,
    offset: usize,
) -> usize {
    let child_id = ctx.next_id();
    block
        .operation
        .push(OperationNode::ChildRef(ChildRefIRNode {
            child_id,
            parent_id,
            offset,
        }));
    child_id
}

fn transform_slot_outlet_child<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    element_id: usize,
    parent_id: usize,
    anchor: usize,
    block: &mut BlockIRNode<'a>,
) {
    let name = get_slot_outlet_name(ctx, el);
    let props = get_slot_outlet_props(ctx, el);
    let fallback = (!el.children.is_empty()).then(|| transform_children(ctx, &el.children));
    block
        .operation
        .push(OperationNode::SlotOutlet(SlotOutletIRNode {
            id: element_id,
            name,
            props,
            fallback,
        }));
    block
        .operation
        .push(OperationNode::InsertNode(InsertNodeIRNode {
            elements: vize_carton::Vec::from_array_in([element_id], &ctx.allocator),
            parent: parent_id,
            anchor: Some(anchor),
        }));
}

fn transform_existing_element<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    element_id: usize,
    block: &mut BlockIRNode<'a>,
) {
    let dynamic_child_count = count_dynamic_element_children(&el.children);

    for prop in el.props.iter() {
        if let PropNode::Directive(dir) = prop {
            transform_directive(ctx, dir, element_id, el, block);
        }
    }

    transform_template_ref(ctx, el, element_id, block);

    transform_text_children(ctx, &el.children, element_id, block);

    let child_ids: std::vec::Vec<usize> = (0..dynamic_child_count).map(|_| ctx.next_id()).collect();
    transform_dynamic_children_with_ids(ctx, el, element_id, block, &child_ids);
}
