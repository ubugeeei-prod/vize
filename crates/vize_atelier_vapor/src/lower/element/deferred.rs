//! Deferred child ID allocation for dynamic and control-flow descendants.

use crate::ir::{ForIRNode, IfIRNode, InsertNodeIRNode, NegativeBranch};
use vize_carton::ensure_sufficient_stack;

use super::component::transform_component;
use super::template::{
    generate_element_template, is_static_element, is_template_backed_element,
    transform_template_ref,
};
use super::{
    BlockIRNode, ChildRefIRNode, ElementNode, ElementType, NextRefIRNode, OperationNode, PropNode,
    SlotOutletIRNode, String, TemplateChildNode, TransformContext, get_slot_outlet_name,
    get_slot_outlet_props, transform_children, transform_directive,
    transform_for_node_deferred_parent, transform_for_node_into_parent,
    transform_if_node_deferred_parent, transform_if_node_into_parent, transform_text_children,
};

/// Transform an element that has control flow children (`v-if`/`v-for`).
///
/// The parent element ID is allocated after direct dynamic children so child
/// refs remain stable while nested control-flow operations can still attach to
/// the parent.
pub(super) fn transform_element_with_control_flow_children<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    block: &mut BlockIRNode<'a>,
) {
    let template = generate_element_template(el);
    let dynamic_child_count = count_dynamic_element_children(&el.children);
    let child_ids: std::vec::Vec<usize> = (0..dynamic_child_count).map(|_| ctx.next_id()).collect();

    if child_ids.is_empty() {
        transform_element_with_deferred_control_flow_parent(ctx, el, block, template);
        return;
    }

    // Allocate the parent after reserving direct dynamic child IDs so child refs
    // still sort before the parent, while keeping all nested wiring anchored to it.
    let element_id = ctx.next_id();

    // Process props and events
    for prop in el.props.iter() {
        match prop {
            PropNode::Directive(dir) => {
                transform_directive(ctx, dir, element_id, el, block);
            }
            PropNode::Attribute(_attr) => {}
        }
    }

    transform_template_ref(ctx, el, element_id, block);

    transform_text_children(ctx, &el.children, element_id, block);

    if !child_ids.is_empty() {
        transform_dynamic_children_with_ids(ctx, el, element_id, block, &child_ids);
    }

    transform_existing_element_control_flow_children(ctx, el, element_id, block);

    // Register template after nested wiring is emitted
    ctx.add_template(element_id, template);

    block.returns.push(element_id);
}

fn transform_element_with_deferred_control_flow_parent<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    block: &mut BlockIRNode<'a>,
    template: String,
) {
    let mut deferred_children = BlockIRNode::new(ctx.allocator);
    transform_deferred_parent_control_flow_children(ctx, el, &mut deferred_children);

    let element_id = ctx.next_id();

    for prop in el.props.iter() {
        match prop {
            PropNode::Directive(dir) => {
                transform_directive(ctx, dir, element_id, el, block);
            }
            PropNode::Attribute(_attr) => {}
        }
    }

    transform_template_ref(ctx, el, element_id, block);

    transform_text_children(ctx, &el.children, element_id, block);

    append_deferred_control_flow_children(block, deferred_children, element_id);

    ctx.add_template(element_id, template);
    block.returns.push(element_id);
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
    let template = generate_element_template(el);

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
    transform_existing_element_control_flow_children(ctx, el, parent_id, block);

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
                transform_slot_outlet_child(ctx, child_el, child_id, parent_id, block);
            } else {
                transform_component(
                    ctx,
                    child_el,
                    block,
                    Some(child_id),
                    Some(parent_id),
                    None,
                    false,
                );
            }
        }

        if is_template_backed_element(child_el) {
            *rendered_index += 1;
            *in_text_run = false;
        }
    }
}

fn transform_slot_outlet_child<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    element_id: usize,
    parent_id: usize,
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
            anchor: None,
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

    if dynamic_child_count != 0 {
        let child_ids: std::vec::Vec<usize> =
            (0..dynamic_child_count).map(|_| ctx.next_id()).collect();
        transform_dynamic_children_with_ids(ctx, el, element_id, block, &child_ids);
    }

    transform_existing_element_control_flow_children(ctx, el, element_id, block);
}

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

fn transform_existing_element_control_flow_children<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    element_id: usize,
    block: &mut BlockIRNode<'a>,
) {
    transform_control_flow_children_into_parent(ctx, &el.children, element_id, block);
}

fn transform_deferred_parent_control_flow_children<'a>(
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

fn append_deferred_control_flow_children<'a>(
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
