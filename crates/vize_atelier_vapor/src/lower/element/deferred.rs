//! Deferred child ID allocation for dynamic and control-flow descendants.

use crate::ir::{InsertNodeIRNode, InsertionAnchor};
use vize_carton::ensure_sufficient_stack;

use super::component::transform_component;
use super::template::{RootAttributes, is_split_child, is_static, transform_template_ref};
use super::{
    BlockIRNode, ChildRefIRNode, ElementNode, ElementType, NextRefIRNode, OperationNode, PropNode,
    SlotOutletIRNode, TemplateChildNode, TransformContext, get_slot_outlet_name,
    get_slot_outlet_props, lower_fallback, transform_directive, transform_for_node_into_parent,
    transform_if_node_into_parent, transform_text_children,
};

/// Transform an element that has control flow children (`v-if`/`v-for`).
///
/// Structural and component children share the same authored insertion anchors.
pub(super) fn transform_element_with_control_flow_children<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    block: &mut BlockIRNode<'a>,
    root: RootAttributes<'a, '_>,
) {
    transform_element_with_dynamic_children(ctx, el, block, root);
}

/// Transform an element that has dynamic element children.
///
/// Child IDs are allocated before the parent ID, and `ChildRef`/`NextRef`
/// operations are used instead of separate templates for each child.
pub(super) fn transform_element_with_dynamic_children<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    block: &mut BlockIRNode<'a>,
    root: RootAttributes<'a, '_>,
) {
    if ctx.split_nesting {
        split_element_with_dynamic_children(ctx, el, block, root);
    } else {
        element_with_dynamic_children::<false>(ctx, el, block, root);
    }
}

#[cold]
#[inline(never)]
fn split_element_with_dynamic_children<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    block: &mut BlockIRNode<'a>,
    root: RootAttributes<'a, '_>,
) {
    element_with_dynamic_children::<true>(ctx, el, block, root);
}

/// `SPLIT`: a child its parent's template string cannot hold is inserted from
/// a template of its own (see [`crate::nesting`]).
fn element_with_dynamic_children<'a, const SPLIT: bool>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    block: &mut BlockIRNode<'a>,
    root: RootAttributes<'a, '_>,
) {
    let dynamic_child_count = count_dynamic_element_children::<SPLIT>(Some(el), &el.children);
    let child_ids: std::vec::Vec<usize> = (0..dynamic_child_count).map(|_| ctx.next_id()).collect();

    // Now allocate parent ID (will be higher than all child IDs)
    let parent_id = ctx.next_id();

    // Generate template (includes all children inline)
    let template = ctx.element_template(root);

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

    transform_dynamic_children_with_ids::<SPLIT>(ctx, el, parent_id, block, &child_ids);

    // Register template for parent
    ctx.add_element_template(parent_id, template, el);

    block.returns.push(parent_id);
}

/// Whether `child` needs an id of its own: it has runtime work, or (under
/// `SPLIT`) `parent` cannot hold it in its template string. `parent` is
/// `None` for the children of a transparent wrapper.
fn needs_child_id<const SPLIT: bool>(
    parent: Option<&ElementNode<'_>>,
    child: &ElementNode<'_>,
) -> bool {
    !is_static::<SPLIT>(child)
        || SPLIT && parent.is_some_and(|parent| is_split_child(parent, child))
}

fn count_dynamic_element_children<const SPLIT: bool>(
    parent: Option<&ElementNode<'_>>,
    children: &[TemplateChildNode<'_>],
) -> usize {
    children
        .iter()
        .map(|child| match child {
            TemplateChildNode::Element(child_el) if child_el.tag_type == ElementType::Template => {
                ensure_sufficient_stack(|| {
                    count_dynamic_element_children::<SPLIT>(None, &child_el.children)
                })
            }
            TemplateChildNode::Element(child_el) if needs_child_id::<SPLIT>(parent, child_el) => 1,
            _ => 0,
        })
        .sum()
}

fn transform_dynamic_children_with_ids<'a, const SPLIT: bool>(
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
    // Comments do not render in Vapor templates; template wrappers flatten
    // into siblings and are never a sole direct child here.
    let sole_child = el
        .children
        .iter()
        .filter(|child| !matches!(child, TemplateChildNode::Comment(_)))
        .count()
        == 1;
    let mut placeholders = super::insertion::block_placeholders::<SPLIT>(
        Some(el),
        &el.children,
        ctx.is_key_non_reactive(),
    )
    .into_iter();
    transform_dynamic_children_in_slice::<SPLIT>(
        ctx,
        Some(el),
        &el.children,
        parent_id,
        block,
        child_ids,
        &mut child_id_index,
        &mut rendered_index,
        &mut in_text_run,
        &mut prev_template_backed_child,
        sole_child,
        &mut placeholders,
    );
    debug_assert_eq!(child_id_index, child_ids.len());
}

#[expect(clippy::too_many_arguments, reason = "the walk's cursors recurse")]
fn transform_dynamic_children_in_slice<'a, const SPLIT: bool>(
    ctx: &mut TransformContext<'a>,
    parent: Option<&ElementNode<'a>>,
    children: &[TemplateChildNode<'a>],
    parent_id: usize,
    block: &mut BlockIRNode<'a>,
    child_ids: &[usize],
    child_id_index: &mut usize,
    rendered_index: &mut usize,
    in_text_run: &mut bool,
    prev_template_backed_child: &mut Option<(usize, usize)>,
    sole_child: bool,
    placeholders: &mut std::vec::IntoIter<bool>,
) {
    for child in vize_atelier_core::walk_probe::vapor_children(children) {
        let TemplateChildNode::Element(child_el) = child else {
            match child {
                TemplateChildNode::If(node) => {
                    let anchor =
                        insertion_anchor(ctx, block, parent_id, *rendered_index, placeholders);
                    transform_if_node_into_parent(ctx, node, block, parent_id, anchor);
                    *rendered_index += 1;
                    *in_text_run = false;
                }
                TemplateChildNode::For(node) => {
                    let anchor =
                        insertion_anchor(ctx, block, parent_id, *rendered_index, placeholders);
                    transform_for_node_into_parent(ctx, node, block, parent_id, anchor, sole_child);
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
            let previous = ctx.non_reactive_keys;
            ctx.non_reactive_keys = super::super::key::is_non_reactive(child_el, previous);
            ensure_sufficient_stack(|| {
                transform_dynamic_children_in_slice::<SPLIT>(
                    ctx,
                    None,
                    &child_el.children,
                    parent_id,
                    block,
                    child_ids,
                    child_id_index,
                    rendered_index,
                    in_text_run,
                    prev_template_backed_child,
                    false,
                    placeholders,
                );
            });
            ctx.non_reactive_keys = previous;
            continue;
        }

        if needs_child_id::<SPLIT>(parent, child_el) {
            // One id was allocated per non-static child, in this order.
            let Some(&child_id) = child_ids.get(*child_id_index) else {
                continue;
            };
            *child_id_index += 1;

            if let Some(value) = super::super::key::value(child_el, ctx.is_key_non_reactive()) {
                let anchor = insertion_anchor(ctx, block, parent_id, *rendered_index, placeholders);
                super::super::key::transform(
                    ctx,
                    child_el,
                    block,
                    Some(child_id),
                    Some(parent_id),
                    Some(anchor),
                    false,
                    value,
                    None,
                );
            } else if SPLIT && parent.is_some_and(|parent| is_split_child(parent, child_el)) {
                let anchor = insertion_anchor(ctx, block, parent_id, *rendered_index, placeholders);
                ensure_sufficient_stack(|| {
                    transform_split_element(ctx, child_el, child_id, parent_id, anchor, block);
                });
            } else if child_el.tag_type == ElementType::Element {
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
                    transform_existing_element::<SPLIT>(ctx, child_el, child_id, block);
                });
            } else if child_el.tag_type == ElementType::Slot {
                let anchor = insertion_anchor(ctx, block, parent_id, *rendered_index, placeholders);
                transform_slot_outlet_child(ctx, child_el, child_id, parent_id, anchor, block);
            } else {
                let anchor = insertion_anchor(ctx, block, parent_id, *rendered_index, placeholders);
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

/// The insertion anchor of the next block child: its template placeholder when
/// a template-rendered sibling follows, otherwise an append at its unit index.
fn insertion_anchor<'a>(
    ctx: &mut TransformContext<'a>,
    block: &mut BlockIRNode<'a>,
    parent_id: usize,
    offset: usize,
    placeholders: &mut std::vec::IntoIter<bool>,
) -> InsertionAnchor {
    if !placeholders.next().unwrap_or(true) {
        return InsertionAnchor::Index(offset);
    }
    let child_id = ctx.next_id();
    block
        .operation
        .push(OperationNode::ChildRef(ChildRefIRNode {
            child_id,
            parent_id,
            offset,
        }));
    InsertionAnchor::Node(child_id)
}

fn transform_slot_outlet_child<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    element_id: usize,
    parent_id: usize,
    anchor: InsertionAnchor,
    block: &mut BlockIRNode<'a>,
) {
    let name = get_slot_outlet_name(ctx, el);
    let (props, has_ref) = get_slot_outlet_props(ctx, el);
    let fallback = lower_fallback(ctx, el);
    block
        .operation
        .push(OperationNode::SlotOutlet(SlotOutletIRNode {
            flags: 0,
            id: element_id,
            name,
            props,
            fallback,
            parent: Some(parent_id),
            anchor: Some(anchor),
        }));
    if has_ref {
        transform_template_ref(ctx, el, element_id, block);
    }
}

/// A child its parent's template string cannot hold, as `@vue/compiler-vapor`
/// emits it: a template of its own, created and inserted into the parent at
/// the block position its authored siblings leave for it.
#[cold]
#[inline(never)]
fn transform_split_element<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    element_id: usize,
    parent_id: usize,
    anchor: InsertionAnchor,
    block: &mut BlockIRNode<'a>,
) {
    let root =
        super::super::key::classify(el, ctx.is_key_non_reactive(), false, true).into_template();
    let template = ctx.element_template(root);
    let mut elements = vize_carton::Vec::new_in(&ctx.allocator);
    elements.push(element_id);
    block
        .operation
        .push(OperationNode::InsertNode(InsertNodeIRNode {
            elements,
            parent: parent_id,
            anchor: match anchor {
                InsertionAnchor::Node(anchor) => Some(anchor),
                InsertionAnchor::Index(_) => None,
            },
        }));
    transform_existing_element::<true>(ctx, el, element_id, block);
    ctx.add_element_template(element_id, template, el);
}

fn transform_existing_element<'a, const SPLIT: bool>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    element_id: usize,
    block: &mut BlockIRNode<'a>,
) {
    let previous = ctx.non_reactive_keys;
    ctx.non_reactive_keys = super::super::key::is_non_reactive(el, previous);
    let dynamic_child_count = count_dynamic_element_children::<SPLIT>(Some(el), &el.children);

    for prop in el.props.iter() {
        if let PropNode::Directive(dir) = prop {
            transform_directive(ctx, dir, element_id, el, block);
        }
    }

    transform_template_ref(ctx, el, element_id, block);

    transform_text_children(ctx, &el.children, element_id, block);

    let child_ids: std::vec::Vec<usize> = (0..dynamic_child_count).map(|_| ctx.next_id()).collect();
    transform_dynamic_children_with_ids::<SPLIT>(ctx, el, element_id, block, &child_ids);
    ctx.non_reactive_keys = previous;
}
