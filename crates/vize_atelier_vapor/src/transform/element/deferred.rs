//! Deferred child ID allocation for dynamic and control-flow descendants.

use super::component::transform_component;
use super::template::*;
use super::*;

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
    let dynamic_child_indices = collect_dynamic_child_indices(el);
    let child_ids: std::vec::Vec<usize> = dynamic_child_indices
        .iter()
        .map(|_| ctx.next_id())
        .collect();

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

    // Handle text content if needed
    let has_text_or_interpolation = el.children.iter().any(|c| {
        matches!(
            c,
            TemplateChildNode::Text(_) | TemplateChildNode::Interpolation(_)
        )
    });
    let has_interpolation = el
        .children
        .iter()
        .any(|c| matches!(c, TemplateChildNode::Interpolation(_)));

    if has_interpolation && has_text_or_interpolation {
        transform_text_children(ctx, &el.children, element_id, block);
    }

    if !dynamic_child_indices.is_empty() {
        transform_dynamic_children_with_ids(
            ctx,
            el,
            element_id,
            block,
            &dynamic_child_indices,
            &child_ids,
        );
    }

    transform_existing_element_control_flow_children(ctx, el, element_id, block);

    // Register template after nested wiring is emitted
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
    let dynamic_child_indices = collect_dynamic_child_indices(el);
    let child_ids: std::vec::Vec<usize> = dynamic_child_indices
        .iter()
        .map(|_| ctx.next_id())
        .collect();

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

    transform_dynamic_children_with_ids(
        ctx,
        el,
        parent_id,
        block,
        &dynamic_child_indices,
        &child_ids,
    );

    // Register template for parent
    ctx.add_template(parent_id, template);

    block.returns.push(parent_id);
}

fn collect_dynamic_child_indices(el: &ElementNode<'_>) -> std::vec::Vec<usize> {
    let mut dynamic_child_indices = std::vec::Vec::new();
    for (i, child) in el.children.iter().enumerate() {
        if let TemplateChildNode::Element(child_el) = child {
            if !is_static_element(child_el) {
                dynamic_child_indices.push(i);
            }
        }
    }
    dynamic_child_indices
}

fn transform_dynamic_children_with_ids<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    parent_id: usize,
    block: &mut BlockIRNode<'a>,
    dynamic_child_indices: &[usize],
    child_ids: &[usize],
) {
    let mut prev_template_backed_child: Option<(usize, usize)> = None;

    for (idx, &child_index) in dynamic_child_indices.iter().enumerate() {
        let child_id = child_ids[idx];
        let TemplateChildNode::Element(child_el) = &el.children[child_index] else {
            continue;
        };

        if is_template_backed_element(child_el) {
            if let Some((prev_child_id, prev_child_index)) = prev_template_backed_child {
                let offset =
                    count_rendered_child_nodes(&el.children, prev_child_index + 1, child_index);
                block.operation.push(OperationNode::NextRef(NextRefIRNode {
                    child_id,
                    prev_id: prev_child_id,
                    offset,
                }));
            } else {
                let offset =
                    count_rendered_child_nodes(&el.children, 0, child_index).saturating_sub(1);
                block
                    .operation
                    .push(OperationNode::ChildRef(ChildRefIRNode {
                        child_id,
                        parent_id,
                        offset,
                    }));
            }

            prev_template_backed_child = Some((child_id, child_index));
            transform_existing_element(ctx, child_el, child_id, block);
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
}

fn transform_existing_element<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    element_id: usize,
    block: &mut BlockIRNode<'a>,
) {
    let has_control_flow_children = el
        .children
        .iter()
        .any(|c| matches!(c, TemplateChildNode::If(_) | TemplateChildNode::For(_)));
    let has_dynamic_element_children = el
        .children
        .iter()
        .any(|c| matches!(c, TemplateChildNode::Element(child_el) if !is_static_element(child_el)));

    for prop in el.props.iter() {
        if let PropNode::Directive(dir) = prop {
            transform_directive(ctx, dir, element_id, el, block);
        }
    }

    transform_template_ref(ctx, el, element_id, block);

    let has_text_or_interpolation = el.children.iter().any(|c| {
        matches!(
            c,
            TemplateChildNode::Text(_) | TemplateChildNode::Interpolation(_)
        )
    });
    let has_interpolation = el
        .children
        .iter()
        .any(|c| matches!(c, TemplateChildNode::Interpolation(_)));

    if has_interpolation && has_text_or_interpolation {
        transform_text_children(ctx, &el.children, element_id, block);
    }

    if has_dynamic_element_children {
        let dynamic_child_indices = collect_dynamic_child_indices(el);
        let child_ids: std::vec::Vec<usize> = dynamic_child_indices
            .iter()
            .map(|_| ctx.next_id())
            .collect();
        transform_dynamic_children_with_ids(
            ctx,
            el,
            element_id,
            block,
            &dynamic_child_indices,
            &child_ids,
        );
    }

    if has_control_flow_children {
        transform_existing_element_control_flow_children(ctx, el, element_id, block);
    }
}

fn transform_existing_element_control_flow_children<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    element_id: usize,
    block: &mut BlockIRNode<'a>,
) {
    for child in el.children.iter() {
        match child {
            TemplateChildNode::If(if_node) => {
                transform_if_node_into_parent(ctx, if_node, block, element_id);
            }
            TemplateChildNode::For(for_node) => {
                transform_for_node_into_parent(ctx, for_node, block, element_id);
            }
            _ => {}
        }
    }
}

fn count_rendered_child_nodes(
    children: &[TemplateChildNode<'_>],
    start: usize,
    end: usize,
) -> usize {
    let mut count = 0usize;
    for child in &children[start..=end] {
        count += count_rendered_nodes_for_child(child);
    }
    count
}

fn count_rendered_nodes_for_child(child: &TemplateChildNode<'_>) -> usize {
    match child {
        TemplateChildNode::Element(child_el) => {
            if child_el.tag_type == ElementType::Template {
                child_el
                    .children
                    .iter()
                    .map(count_rendered_nodes_for_child)
                    .sum()
            } else if is_template_backed_element(child_el) {
                1
            } else {
                0
            }
        }
        TemplateChildNode::Text(_) | TemplateChildNode::Interpolation(_) => 1,
        _ => 0,
    }
}
