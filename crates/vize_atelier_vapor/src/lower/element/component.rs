//! Component, slot outlet, dynamic component, and v-model lowering.

use super::{
    BlockIRNode, Box, ComponentKind, CreateComponentIRNode, ElementNode, ElementType,
    ExpressionNode, IRProp, IRSlot, OperationNode, PropNode, SimpleExpressionNode, SourceLocation,
    String, TemplateChildNode, TransformContext, Vec, transform_children,
};

mod model;
mod slots;
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
    anchor: Option<usize>,
    add_return: bool,
) {
    let tag = el.tag;
    let kind = match tag {
        "Teleport" => ComponentKind::Teleport,
        "KeepAlive" => ComponentKind::KeepAlive,
        "Suspense" => ComponentKind::Suspense,
        "component" => ComponentKind::Dynamic,
        _ => ComponentKind::Regular,
    };

    let mut props = Vec::new_in(&ctx.allocator);
    let mut slots = Vec::new_in(&ctx.allocator);
    let mut v_show_exp: Option<Box<'a, SimpleExpressionNode<'a>>> = None;
    let mut is_expr: Option<Box<'a, SimpleExpressionNode<'a>>> = None;
    let mut has_dynamic_slot = false;

    // Check for v-slot on the component itself (scoped default slot)
    let mut has_v_slot_on_component = false;
    let mut slot_props_expr: Option<String> = None;
    for prop in el.props.iter() {
        if let PropNode::Directive(dir) = prop
            && dir.name == "slot"
        {
            has_v_slot_on_component = true;
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
                            if kind == ComponentKind::Dynamic && key_exp.content == "is" {
                                if let Some(ref exp) = dir.exp
                                    && let ExpressionNode::Simple(val_exp) = exp
                                {
                                    let node = SimpleExpressionNode::from_node(val_exp);
                                    is_expr = Some(Box::new_in(node, &ctx.allocator));
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
                            props.push(IRProp {
                                key,
                                values,
                                is_component: true,
                            });
                        }
                    } else if let Some(ref exp) = dir.exp
                        && let ExpressionNode::Simple(val_exp) = exp
                    {
                        let key_node = SimpleExpressionNode::new("$", true, SourceLocation::STUB);
                        let key = Box::new_in(key_node, &ctx.allocator);
                        let mut values = Vec::new_in(&ctx.allocator);
                        let val_node = SimpleExpressionNode::from_node(val_exp);
                        values.push(Box::new_in(val_node, &ctx.allocator));
                        props.push(IRProp {
                            key,
                            values,
                            is_component: true,
                        });
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
                        props.push(IRProp {
                            key,
                            values,
                            is_component: true,
                        });
                    }
                } else if dir.name == "model" {
                    transform_component_v_model(ctx, dir, &mut props);
                }
            }
            PropNode::Attribute(attr) => {
                if attr.name == "key" {
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
                props.push(IRProp {
                    key,
                    values,
                    is_component: true,
                });
            }
        }
    }

    // Process children to create slots
    if has_v_slot_on_component {
        let slot_block = transform_children(ctx, &el.children);
        let name_exp = SimpleExpressionNode::new("default", true, SourceLocation::STUB);
        let fn_exp = slot_props_expr.map(|expr| {
            let text = ctx.allocator.alloc_str(&expr);
            let node = SimpleExpressionNode::new(text, false, SourceLocation::STUB);
            Box::new_in(node, &ctx.allocator)
        });
        slots.push(IRSlot {
            name: Box::new_in(name_exp, &ctx.allocator),
            fn_exp,
            block: slot_block,
        });
    } else if !el.children.is_empty() {
        let has_named_slots = el.children.iter().any(|c| {
            if let TemplateChildNode::Element(child_el) = c {
                child_el.tag_type == ElementType::Template
                    && child_el
                        .props
                        .iter()
                        .any(|p| matches!(p, PropNode::Directive(d) if d.name == "slot"))
            } else {
                false
            }
        });

        if has_named_slots {
            for child in el.children.iter() {
                if let TemplateChildNode::Element(child_el) = child
                    && child_el.tag_type == ElementType::Template
                {
                    for prop in child_el.props.iter() {
                        if let PropNode::Directive(dir) = prop
                            && dir.name == "slot"
                        {
                            let (slot_name, is_static_name) = slots::resolve_named_slot(dir);
                            if !is_static_name {
                                has_dynamic_slot = true;
                            }
                            let fn_exp = dir.exp.as_ref().and_then(|exp| match exp {
                                ExpressionNode::Simple(s) => {
                                    let node = SimpleExpressionNode::new(
                                        s.content,
                                        false,
                                        SourceLocation::STUB,
                                    );
                                    Some(Box::new_in(node, &ctx.allocator))
                                }
                                _ => None,
                            });
                            let slot_block = transform_children(ctx, &child_el.children);
                            let _template_id = ctx.next_id(); // consume ID for template wrapper
                            let n = ctx.allocator.alloc_str(&slot_name);
                            let name_exp =
                                SimpleExpressionNode::new(n, is_static_name, SourceLocation::STUB);
                            slots.push(IRSlot {
                                name: Box::new_in(name_exp, &ctx.allocator),
                                fn_exp,
                                block: slot_block,
                            });
                        }
                    }
                }
            }
        } else {
            let slot_block = transform_children(ctx, &el.children);
            let name_exp = SimpleExpressionNode::new("default", true, SourceLocation::STUB);
            slots.push(IRSlot {
                name: Box::new_in(name_exp, &ctx.allocator),
                fn_exp: None,
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
        asset: kind == ComponentKind::Regular || kind == ComponentKind::Suspense,
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
    if add_return {
        block.returns.push(element_id);
    }
}
