//! Slot detection predicates (which children form slots, dynamic/forwarded checks).

use crate::codegen::context::CodegenContext;
use crate::steps::v_slot::{collect_slots, has_v_slot};
use crate::{ElementNode, ElementType, ExpressionNode, PropNode, TemplateChildNode};
use vize_s0::ensure_sufficient_stack;

/// The expression a component spreads into its slots object, if it carries a
/// `v-slots` directive (#3467).
///
/// `v-slots` is the `@vue/babel-plugin-jsx` spelling for slot forwarding:
/// `<B v-slots={slots}/>` hands `B` a slots object the compiler cannot see
/// inside. The JSX lowering keeps the object-literal form as synthetic
/// `<template v-slot:name>` children (#3418) and only reaches here for a value
/// that stays opaque at compile time.
pub(super) fn slots_spread<'a, 'b>(el: &'b ElementNode<'a>) -> Option<&'b ExpressionNode<'a>> {
    el.props.iter().find_map(|prop| {
        let PropNode::Directive(dir) = prop else {
            return None;
        };
        // `v-slots` takes no argument -- the slot names are the forwarded
        // object's own keys -- so an argument spelling is not this construct.
        if dir.name != "slots" || dir.arg.is_some() {
            return None;
        }
        dir.exp.as_ref()
    })
}

/// Whether the component's slots object is built **only** from a forwarded
/// `v-slots` value: no `v-slot` on the component root, no slot templates, and
/// no children to become the default slot.
///
/// `@vue/babel-plugin-jsx` passes the forwarded value straight through as the
/// children argument in that shape (`createVNode(B, null, slots)`) rather than
/// wrapping it in an object literal, and Vize matches it.
pub(super) fn slots_are_only_forwarded(el: &ElementNode<'_>) -> bool {
    slots_spread(el).is_some() && !has_authored_slots(el)
}

/// Whether the element contributes slots of its own: a `v-slot` on the
/// component root, or any child that is not whitespace/comment filler.
fn has_authored_slots(el: &ElementNode<'_>) -> bool {
    if el
        .props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(dir) if dir.name == "slot"))
    {
        return true;
    }
    el.children.iter().any(|child| match child {
        TemplateChildNode::Text(t) => !t.content.trim().is_empty(),
        TemplateChildNode::Comment(_) => false,
        _ => true,
    })
}

/// Check if component has slot children that need to be generated as slots object
pub fn has_slot_children(el: &ElementNode<'_>) -> bool {
    // Teleport and KeepAlive consume raw children rather than a slot object.
    // KeepAlive still gets DYNAMIC_SLOTS at the vnode patch-flag layer.
    if matches!(el.tag, "Teleport" | "teleport" | "KeepAlive" | "keep-alive") {
        return false;
    }

    // A forwarded `v-slots` value is the component's slots even when it has no
    // children of its own (`<B v-slots={slots}/>`).
    if slots_spread(el).is_some() {
        return true;
    }

    if el.children.is_empty() {
        return false;
    }

    // Check for v-slot on component root
    for prop in &el.props {
        if let PropNode::Directive(dir) = prop
            && dir.name == "slot"
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

/// KeepAlive always needs `DYNAMIC_SLOTS`. Other components need it when the
/// slot object itself can change (`v-for` parent, dynamic names, forwarded
/// outlets).
pub fn needs_dynamic_slots_patch(ctx: &CodegenContext, el: &ElementNode<'_>) -> bool {
    el.tag == "KeepAlive"
        || el.tag == "keep-alive"
        || (ctx.in_v_for && has_slot_children(el))
        || has_dynamic_slots_flag(el, &ctx.source)
        || (ctx.has_slot_params() && has_forwarded_slot_outlet(el))
}

/// Check if component has dynamic slots (requires DYNAMIC_SLOTS patch flag)
pub fn has_dynamic_slots_flag(el: &ElementNode<'_>, source: &str) -> bool {
    // A forwarded slots object can change without anything on this vnode
    // changing, and the emitted slots object carries no `_` stability flag (see
    // `generate_slots`), so the child is only re-rendered if the parent forces
    // it through DYNAMIC_SLOTS. `@vue/babel-plugin-jsx` gets the same forced
    // update for free by emitting no patch flags at all: `shouldUpdateComponent`
    // falls back to "any children means update" for unoptimized vnodes.
    if slots_spread(el).is_some() {
        return true;
    }
    let collected_slots = collect_slots(el, source);
    if collected_slots.iter().any(|s| s.is_dynamic) {
        return true;
    }
    // Also check for v-if/v-for on slot templates (they become IfNode/ForNode children)
    has_conditional_or_loop_slots(el)
}

/// Check whether this component forwards an incoming slot to another component,
/// e.g. `<Inner><slot /></Inner>`.
pub fn has_forwarded_slot_outlet(el: &ElementNode<'_>) -> bool {
    el.children.iter().any(child_contains_slot_outlet)
}

fn child_contains_slot_outlet(child: &TemplateChildNode<'_>) -> bool {
    match child {
        TemplateChildNode::Element(el) => {
            if el.tag_type == ElementType::Slot || el.tag == "slot" {
                return true;
            }
            ensure_sufficient_stack(|| el.children.iter().any(child_contains_slot_outlet))
        }
        TemplateChildNode::If(if_node) => if_node
            .branches
            .iter()
            .flat_map(|branch| branch.children.iter())
            .any(|child| ensure_sufficient_stack(|| child_contains_slot_outlet(child))),
        TemplateChildNode::For(for_node) => for_node
            .children
            .iter()
            .any(|child| ensure_sufficient_stack(|| child_contains_slot_outlet(child))),
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
                    el.tag == "template" && has_v_slot(el)
                } else {
                    false
                }
            })
        }),
        TemplateChildNode::For(for_node) => for_node.children.iter().any(|c| {
            if let TemplateChildNode::Element(el) = c {
                el.tag == "template" && has_v_slot(el)
            } else {
                false
            }
        }),
        _ => false,
    })
}

pub(super) fn child_is_slot_template(child: &TemplateChildNode<'_>) -> bool {
    match child {
        TemplateChildNode::Element(el) => el.tag == "template" && has_v_slot(el),
        TemplateChildNode::If(if_node) => if_node.branches.iter().any(|branch| {
            branch.children.iter().any(|child| {
                matches!(
                    child,
                    TemplateChildNode::Element(el)
                        if el.tag == "template" && has_v_slot(el)
                )
            })
        }),
        TemplateChildNode::For(for_node) => for_node.children.iter().any(|child| {
            matches!(
                child,
                TemplateChildNode::Element(el)
                    if el.tag == "template" && has_v_slot(el)
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
