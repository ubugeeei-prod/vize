//! Component, slot outlet, dynamic component, and v-model lowering.

use super::{
    BlockIRNode, Box, ComponentKind, CreateComponentIRNode, ElementNode, ExpressionNode, IRProp,
    IRSlot, OperationNode, PropNode, SimpleExpressionNode, SourceLocation, String,
    TransformContext, Vec, transform_children,
};

mod model;
mod slots;
mod structural_slots;
use model::transform_component_v_model;

/// Transform a component element into a `CreateComponent` operation.
///
/// This handles slot collection, built-in component kinds, dynamic components,
/// parent anchors, and `v-show` metadata in one pass so ID allocation remains
/// deterministic.
pub(super) fn transform_component<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    block: &mut BlockIRNode<'a>,
    existing_id: Option<usize>,
    parent: Option<usize>,
    anchor: Option<crate::ir::InsertionAnchor>,
    add_return: bool,
) {
    let tag = el.tag;
    let kind = match tag {
        "Teleport" => ComponentKind::Teleport,
        "KeepAlive" => ComponentKind::KeepAlive,
        "Suspense" => ComponentKind::Suspense,
        "Transition" => ComponentKind::Transition,
        "TransitionGroup" => ComponentKind::TransitionGroup,
        "component" => ComponentKind::Dynamic,
        _ => ComponentKind::Regular,
    };

    let mut props = Vec::new_in(&ctx.allocator);
    let mut slots = Vec::new_in(&ctx.allocator);
    let mut v_show_exp: Option<Box<'a, SimpleExpressionNode<'a>>> = None;
    let mut is_expr: Option<Box<'a, SimpleExpressionNode<'a>>> = None;
    let mut is_selected = false;
    let mut has_dynamic_slot = false;

    // Check for v-slot on the component itself (named or default slot)
    let mut has_v_slot_on_component = false;
    let mut slot_props_expr: Option<String> = None;
    let mut own_slot_name = SimpleExpressionNode::new("default", true, SourceLocation::STUB);
    for prop in el.props.iter() {
        if let PropNode::Directive(dir) = prop
            && dir.name == "slot"
        {
            has_v_slot_on_component = true;
            let (name, is_static) = slots::resolve_named_slot(dir);
            own_slot_name = SimpleExpressionNode::new(
                ctx.allocator.alloc_str(&name),
                is_static,
                dir.arg
                    .as_ref()
                    .map_or(SourceLocation::STUB, |arg| arg.loc().clone()),
            );
            has_dynamic_slot |= !is_static;
            if let Some(ref exp) = dir.exp
                && let ExpressionNode::Simple(s) = exp
            {
                slot_props_expr = Some(s.content.into());
            }
        }
    }

    // Process props
    for prop in el.props.iter() {
        match prop {
            PropNode::Directive(dir) => {
                if dir.name == "slot" {
                    continue;
                }
                if dir.name == "show" {
                    if let Some(ref exp) = dir.exp
                        && let ExpressionNode::Simple(s) = exp
                    {
                        let node = SimpleExpressionNode::from_node(s);
                        v_show_exp = Some(Box::new_in(node, &ctx.allocator));
                    }
                    continue;
                }
                if dir.name == "bind" {
                    if let Some(ref arg) = dir.arg {
                        if let ExpressionNode::Simple(key_exp) = arg {
                            if key_exp.is_static && key_exp.content == "key" {
                                continue;
                            }
                            if kind == ComponentKind::Dynamic
                                && key_exp.is_static
                                && key_exp.content == "is"
                            {
                                if !is_selected {
                                    is_selected = true;
                                    if let Some(ref exp) = dir.exp
                                        && let ExpressionNode::Simple(val_exp) = exp
                                    {
                                        let node = SimpleExpressionNode::from_node(val_exp);
                                        is_expr = Some(Box::new_in(node, &ctx.allocator));
                                    }
                                }
                                continue;
                            }
                            let key_node = SimpleExpressionNode::from_node(key_exp);
                            let key = Box::new_in(key_node, &ctx.allocator);
                            let mut values = Vec::new_in(&ctx.allocator);
                            if let Some(ref exp) = dir.exp
                                && let ExpressionNode::Simple(val_exp) = exp
                            {
                                let val_node = SimpleExpressionNode::from_node(val_exp);
                                values.push(Box::new_in(val_node, &ctx.allocator));
                            }
                            props.push(IRProp::new(key, values, true));
                        }
                    } else if let Some(ref exp) = dir.exp
                        && let ExpressionNode::Simple(val_exp) = exp
                    {
                        let key_node = SimpleExpressionNode::new("$", true, SourceLocation::STUB);
                        let key = Box::new_in(key_node, &ctx.allocator);
                        let mut values = Vec::new_in(&ctx.allocator);
                        let val_node = SimpleExpressionNode::from_node(val_exp);
                        values.push(Box::new_in(val_node, &ctx.allocator));
                        props.push(IRProp::new(key, values, true));
                    }
                } else if dir.name == "on" {
                    if let Some(ref arg) = dir.arg
                        && let ExpressionNode::Simple(event_exp) = arg
                    {
                        let mut key_node = if event_exp.is_static {
                            let event_name = event_exp.content;
                            let on_name = if event_name.is_empty() {
                                "on"
                            } else {
                                let mut s = String::from("on");
                                let mut chars = event_name.chars();
                                if let Some(c) = chars.next() {
                                    s.push(c.to_ascii_uppercase());
                                }
                                for c in chars {
                                    s.push(c);
                                }
                                ctx.allocator.alloc_str(&s)
                            };
                            SimpleExpressionNode::new(on_name, true, event_exp.loc.clone())
                        } else {
                            SimpleExpressionNode::from_node(event_exp)
                        };
                        key_node.is_handler_key = true;
                        let key = Box::new_in(key_node, &ctx.allocator);
                        let mut values = Vec::new_in(&ctx.allocator);
                        if let Some(ref exp) = dir.exp
                            && let ExpressionNode::Simple(val_exp) = exp
                        {
                            let val_node = SimpleExpressionNode::from_node(val_exp);
                            values.push(Box::new_in(val_node, &ctx.allocator));
                        }
                        props.push(IRProp::new(key, values, true));
                    }
                } else if dir.name == "model" {
                    transform_component_v_model(ctx, dir, &mut props);
                }
            }
            PropNode::Attribute(attr) => {
                if attr.name == "key" {
                    continue;
                }
                // `<component is="a">` names its component statically; it is
                // resolved once instead of being forwarded as a prop.
                if kind == ComponentKind::Dynamic && attr.name == "is" {
                    if !is_selected {
                        is_selected = true;
                        let value = attr.value.as_ref().map_or("", |value| value.content);
                        let node = SimpleExpressionNode::new(value, true, SourceLocation::STUB);
                        is_expr = Some(Box::new_in(node, &ctx.allocator));
                    }
                    continue;
                }
                let key_node = SimpleExpressionNode::new(attr.name, true, SourceLocation::STUB);
                let key = Box::new_in(key_node, &ctx.allocator);
                let mut values = Vec::new_in(&ctx.allocator);
                if let Some(ref value) = attr.value {
                    let val_node =
                        SimpleExpressionNode::new(value.content, true, SourceLocation::STUB);
                    values.push(Box::new_in(val_node, &ctx.allocator));
                }
                props.push(IRProp::new(key, values, true));
            }
        }
    }

    // Process children to create slots
    if has_v_slot_on_component {
        let slot_block = transform_children(ctx, &el.children);
        let fn_exp = slot_props_expr.map(|expr| {
            let text = ctx.allocator.alloc_str(&expr);
            let node = SimpleExpressionNode::new(text, false, SourceLocation::STUB);
            Box::new_in(node, &ctx.allocator)
        });
        slots.push(IRSlot {
            name: Box::new_in(own_slot_name, &ctx.allocator),
            fn_exp,
            control: None,
            block: slot_block,
        });
    } else if !el.children.is_empty() {
        let has_named_slots = el.children.iter().any(structural_slots::is_slot);
        if has_named_slots {
            for child in &el.children {
                if let Some(slot) = structural_slots::lower(ctx, child) {
                    has_dynamic_slot |= slot.dynamic();
                    slots.push(slot);
                }
            }
        } else {
            let slot_block = transform_children(ctx, &el.children);
            let name_exp = SimpleExpressionNode::new("default", true, SourceLocation::STUB);
            slots.push(IRSlot {
                name: Box::new_in(name_exp, &ctx.allocator),
                fn_exp: None,
                control: None,
                block: slot_block,
            });
        }
    }

    let element_id = existing_id.unwrap_or_else(|| ctx.next_id());

    let create_component = CreateComponentIRNode {
        id: element_id,
        tag: el.tag,
        props,
        slots,
        asset: kind == ComponentKind::Regular,
        once: false,
        dynamic_slots: has_dynamic_slot,
        kind,
        is_expr,
        v_show: v_show_exp,
        parent,
        anchor,
    };

    block
        .operation
        .push(OperationNode::CreateComponent(create_component));
    for prop in &el.props {
        if let PropNode::Directive(dir) = prop
            && !matches!(
                dir.name,
                "bind"
                    | "on"
                    | "model"
                    | "slot"
                    | "show"
                    | "once"
                    | "memo"
                    | "cloak"
                    | "pre"
                    | "if"
                    | "else"
                    | "else-if"
                    | "for"
                    | "text"
                    | "html"
            )
        {
            super::super::directive::transform_directive(ctx, dir, element_id, el, block);
        }
    }
    if add_return {
        block.returns.push(element_id);
    }
}
