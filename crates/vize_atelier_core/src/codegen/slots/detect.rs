//! Slot detection predicates (which children form slots, dynamic/forwarded checks).

use crate::transforms::v_slot::{collect_slots, has_v_slot};
use crate::*;

/// Check if component has slot children that need to be generated as slots object
pub fn has_slot_children(el: &ElementNode<'_>) -> bool {
    if el.children.is_empty() {
        return false;
    }

    // Teleport and KeepAlive consume raw children rather than a slot object.
    // KeepAlive still gets DYNAMIC_SLOTS at the vnode patch-flag layer.
    if matches!(
        el.tag.as_str(),
        "Teleport" | "teleport" | "KeepAlive" | "keep-alive"
    ) {
        return false;
    }

    // Check for v-slot on component root
    for prop in &el.props {
        if let PropNode::Directive(dir) = prop
            && dir.name.as_str() == "slot"
        {
            return true;
        }
    }

    // If children consist only of whitespace text and/or comments, skip slot generation.
    // This matches Vue's official compiler behavior where `<Comp> </Comp>` does not
    // produce a default slot (important for <router-view>, <transition>, etc.).
    let has_meaningful_child = el.children.iter().any(|child| match child {
        TemplateChildNode::Text(t) => !t.content.trim().is_empty(),
        TemplateChildNode::Comment(_) => false,
        _ => true,
    });
    if !has_meaningful_child {
        return false;
    }

    // Check for any children (default slot) or template slots
    true
}

/// Check if component has dynamic slots (requires DYNAMIC_SLOTS patch flag)
pub fn has_dynamic_slots_flag(el: &ElementNode<'_>) -> bool {
    let collected_slots = collect_slots(el);
    if collected_slots.iter().any(|s| s.is_dynamic) {
        return true;
    }
    if has_forwarded_slot_outlet(el) {
        return true;
    }
    // Also check for v-if/v-for on slot templates (they become IfNode/ForNode children)
    has_conditional_or_loop_slots(el)
}

/// Check whether this component forwards an incoming slot to another component,
/// e.g. `<Inner><slot /></Inner>`.
pub(super) fn has_forwarded_slot_outlet(el: &ElementNode<'_>) -> bool {
    el.children.iter().any(child_contains_slot_outlet)
}

fn child_contains_slot_outlet(child: &TemplateChildNode<'_>) -> bool {
    match child {
        TemplateChildNode::Element(el) => {
            if el.tag_type == ElementType::Slot || el.tag.as_str() == "slot" {
                return true;
            }
            el.children.iter().any(child_contains_slot_outlet)
        }
        TemplateChildNode::If(if_node) => if_node
            .branches
            .iter()
            .flat_map(|branch| branch.children.iter())
            .any(child_contains_slot_outlet),
        TemplateChildNode::For(for_node) => {
            for_node.children.iter().any(child_contains_slot_outlet)
        }
        _ => false,
    }
}

/// Check if children have conditional (v-if) or looped (v-for) slot templates.
/// Only returns true when the IfNode/ForNode wraps a `<template v-slot>` element.
pub(super) fn has_conditional_or_loop_slots(el: &ElementNode<'_>) -> bool {
    el.children.iter().any(|child| match child {
        TemplateChildNode::If(if_node) => if_node.branches.iter().any(|branch| {
            branch.children.iter().any(|c| {
                if let TemplateChildNode::Element(el) = c {
                    el.tag.as_str() == "template" && has_v_slot(el)
                } else {
                    false
                }
            })
        }),
        TemplateChildNode::For(for_node) => for_node.children.iter().any(|c| {
            if let TemplateChildNode::Element(el) = c {
                el.tag.as_str() == "template" && has_v_slot(el)
            } else {
                false
            }
        }),
        _ => false,
    })
}

pub(super) fn child_is_slot_template(child: &TemplateChildNode<'_>) -> bool {
    match child {
        TemplateChildNode::Element(el) => el.tag.as_str() == "template" && has_v_slot(el),
        TemplateChildNode::If(if_node) => if_node.branches.iter().any(|branch| {
            branch.children.iter().any(|child| {
                matches!(
                    child,
                    TemplateChildNode::Element(el)
                        if el.tag.as_str() == "template" && has_v_slot(el)
                )
            })
        }),
        TemplateChildNode::For(for_node) => for_node.children.iter().any(|child| {
            matches!(
                child,
                TemplateChildNode::Element(el)
                    if el.tag.as_str() == "template" && has_v_slot(el)
            )
        }),
        _ => false,
    }
}

pub(super) fn slot_children_have_meaningful_content(children: &[&TemplateChildNode<'_>]) -> bool {
    children.iter().any(|child| match child {
        TemplateChildNode::Text(text) => !text.content.trim().is_empty(),
        TemplateChildNode::Comment(_) => false,
        _ => true,
    })
}
