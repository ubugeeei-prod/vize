//! Component, slot outlet, dynamic component, and v-model lowering.

use super::*;

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
    let tag = el.tag.as_str();
    let kind = match tag {
        "Teleport" => ComponentKind::Teleport,
        "KeepAlive" => ComponentKind::KeepAlive,
        "Suspense" => ComponentKind::Suspense,
        "component" => ComponentKind::Dynamic,
        _ => ComponentKind::Regular,
    };

    let mut props = Vec::new_in(ctx.allocator);
    let mut slots = Vec::new_in(ctx.allocator);
    let mut v_show_exp: Option<Box<'a, SimpleExpressionNode<'a>>> = None;
    let mut is_expr: Option<Box<'a, SimpleExpressionNode<'a>>> = None;
    let mut has_dynamic_slot = false;

    // Check for v-slot on the component itself (scoped default slot)
    let mut has_v_slot_on_component = false;
    let mut slot_props_expr: Option<String> = None;
    for prop in el.props.iter() {
        if let PropNode::Directive(dir) = prop {
            if dir.name.as_str() == "slot" {
                has_v_slot_on_component = true;
                if let Some(ref exp) = dir.exp {
                    if let ExpressionNode::Simple(s) = exp {
                        slot_props_expr = Some(s.content.clone());
                    }
                }
            }
        }
    }

    // Process props
    for prop in el.props.iter() {
        match prop {
            PropNode::Directive(dir) => {
                if dir.name.as_str() == "slot" {
                    continue;
                }
                if dir.name.as_str() == "show" {
                    if let Some(ref exp) = dir.exp {
                        if let ExpressionNode::Simple(s) = exp {
                            let node = SimpleExpressionNode::new(
                                s.content.clone(),
                                s.is_static,
                                s.loc.clone(),
                            );
                            v_show_exp = Some(Box::new_in(node, ctx.allocator));
                        }
                    }
                    continue;
                }
                if dir.name.as_str() == "bind" {
                    if let Some(ref arg) = dir.arg {
                        if let ExpressionNode::Simple(key_exp) = arg {
                            if kind == ComponentKind::Dynamic && key_exp.content.as_str() == "is" {
                                if let Some(ref exp) = dir.exp {
                                    if let ExpressionNode::Simple(val_exp) = exp {
                                        let node = SimpleExpressionNode::new(
                                            val_exp.content.clone(),
                                            val_exp.is_static,
                                            val_exp.loc.clone(),
                                        );
                                        is_expr = Some(Box::new_in(node, ctx.allocator));
                                    }
                                }
                                continue;
                            }
                            let key_node = SimpleExpressionNode::new(
                                key_exp.content.clone(),
                                key_exp.is_static,
                                key_exp.loc.clone(),
                            );
                            let key = Box::new_in(key_node, ctx.allocator);
                            let mut values = Vec::new_in(ctx.allocator);
                            if let Some(ref exp) = dir.exp {
                                if let ExpressionNode::Simple(val_exp) = exp {
                                    let val_node = SimpleExpressionNode::new(
                                        val_exp.content.clone(),
                                        val_exp.is_static,
                                        val_exp.loc.clone(),
                                    );
                                    values.push(Box::new_in(val_node, ctx.allocator));
                                }
                            }
                            props.push(IRProp {
                                key,
                                values,
                                is_component: true,
                            });
                        }
                    } else if let Some(ref exp) = dir.exp {
                        if let ExpressionNode::Simple(val_exp) = exp {
                            let key_node =
                                SimpleExpressionNode::new("$", true, SourceLocation::STUB);
                            let key = Box::new_in(key_node, ctx.allocator);
                            let mut values = Vec::new_in(ctx.allocator);
                            let val_node = SimpleExpressionNode::new(
                                val_exp.content.clone(),
                                val_exp.is_static,
                                val_exp.loc.clone(),
                            );
                            values.push(Box::new_in(val_node, ctx.allocator));
                            props.push(IRProp {
                                key,
                                values,
                                is_component: true,
                            });
                        }
                    }
                } else if dir.name.as_str() == "on" {
                    if let Some(ref arg) = dir.arg {
                        if let ExpressionNode::Simple(event_exp) = arg {
                            let event_name = event_exp.content.as_str();
                            let on_name = if event_name.is_empty() {
                                String::from("on")
                            } else {
                                let mut s = String::from("on");
                                let mut chars = event_name.chars();
                                if let Some(c) = chars.next() {
                                    s.push(c.to_ascii_uppercase());
                                }
                                for c in chars {
                                    s.push(c);
                                }
                                s
                            };
                            let key_node =
                                SimpleExpressionNode::new(on_name, true, event_exp.loc.clone());
                            let key = Box::new_in(key_node, ctx.allocator);
                            let mut values = Vec::new_in(ctx.allocator);
                            if let Some(ref exp) = dir.exp {
                                if let ExpressionNode::Simple(val_exp) = exp {
                                    let val_node = SimpleExpressionNode::new(
                                        val_exp.content.clone(),
                                        val_exp.is_static,
                                        val_exp.loc.clone(),
                                    );
                                    values.push(Box::new_in(val_node, ctx.allocator));
                                }
                            }
                            props.push(IRProp {
                                key,
                                values,
                                is_component: true,
                            });
                        }
                    }
                } else if dir.name.as_str() == "model" {
                    transform_component_v_model(ctx, dir, &mut props);
                }
            }
            PropNode::Attribute(attr) => {
                let key_node =
                    SimpleExpressionNode::new(attr.name.clone(), true, SourceLocation::STUB);
                let key = Box::new_in(key_node, ctx.allocator);
                let mut values = Vec::new_in(ctx.allocator);
                if let Some(ref value) = attr.value {
                    let val_node = SimpleExpressionNode::new(
                        value.content.clone(),
                        true,
                        SourceLocation::STUB,
                    );
                    values.push(Box::new_in(val_node, ctx.allocator));
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
            let node = SimpleExpressionNode::new(expr, false, SourceLocation::STUB);
            Box::new_in(node, ctx.allocator)
        });
        slots.push(IRSlot {
            name: Box::new_in(name_exp, ctx.allocator),
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
                        .any(|p| matches!(p, PropNode::Directive(d) if d.name.as_str() == "slot"))
            } else {
                false
            }
        });

        if has_named_slots {
            for child in el.children.iter() {
                if let TemplateChildNode::Element(child_el) = child {
                    if child_el.tag_type == ElementType::Template {
                        for prop in child_el.props.iter() {
                            if let PropNode::Directive(dir) = prop {
                                if dir.name.as_str() == "slot" {
                                    let (slot_name, is_static_name) = if let Some(ref arg) = dir.arg
                                    {
                                        match arg {
                                            ExpressionNode::Simple(exp) => {
                                                (exp.content.clone(), exp.is_static)
                                            }
                                            _ => (String::from("default"), true),
                                        }
                                    } else {
                                        (String::from("default"), true)
                                    };
                                    if !is_static_name {
                                        has_dynamic_slot = true;
                                    }
                                    let fn_exp = dir.exp.as_ref().and_then(|exp| match exp {
                                        ExpressionNode::Simple(s) => {
                                            let node = SimpleExpressionNode::new(
                                                s.content.clone(),
                                                false,
                                                SourceLocation::STUB,
                                            );
                                            Some(Box::new_in(node, ctx.allocator))
                                        }
                                        _ => None,
                                    });
                                    let slot_block = transform_children(ctx, &child_el.children);
                                    let _template_id = ctx.next_id(); // consume ID for template wrapper
                                    let name_exp = SimpleExpressionNode::new(
                                        slot_name,
                                        is_static_name,
                                        SourceLocation::STUB,
                                    );
                                    slots.push(IRSlot {
                                        name: Box::new_in(name_exp, ctx.allocator),
                                        fn_exp,
                                        block: slot_block,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        } else {
            let slot_block = transform_children(ctx, &el.children);
            let name_exp = SimpleExpressionNode::new("default", true, SourceLocation::STUB);
            slots.push(IRSlot {
                name: Box::new_in(name_exp, ctx.allocator),
                fn_exp: None,
                block: slot_block,
            });
        }
    }

    let element_id = existing_id.unwrap_or_else(|| ctx.next_id());

    let create_component = CreateComponentIRNode {
        id: element_id,
        tag: el.tag.clone(),
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

/// Transform v-model on component (helper for transform_component)
pub(super) fn transform_component_v_model<'a>(
    ctx: &mut TransformContext<'a>,
    dir: &vize_atelier_core::DirectiveNode<'a>,
    props: &mut Vec<'a, IRProp<'a>>,
) {
    let binding = if let Some(ref exp) = dir.exp {
        match exp {
            ExpressionNode::Simple(s) => s.content.clone(),
            _ => String::from(""),
        }
    } else {
        String::from("")
    };
    let prop_name = dir
        .arg
        .as_ref()
        .map(|arg| match arg {
            ExpressionNode::Simple(s) => s.content.clone(),
            _ => String::from("modelValue"),
        })
        .unwrap_or_else(|| String::from("modelValue"));

    let key_node = SimpleExpressionNode::new(prop_name.clone(), true, SourceLocation::STUB);
    let key = Box::new_in(key_node, ctx.allocator);
    let mut values = Vec::new_in(ctx.allocator);
    let val_node = SimpleExpressionNode::new(binding.clone(), false, SourceLocation::STUB);
    values.push(Box::new_in(val_node, ctx.allocator));
    props.push(IRProp {
        key,
        values,
        is_component: true,
    });

    let event_key = {
        let mut s = String::from("onUpdate:");
        s.push_str(prop_name.as_str());
        s
    };
    let event_key_node = SimpleExpressionNode::new(event_key, true, SourceLocation::STUB);
    let event_key_box = Box::new_in(event_key_node, ctx.allocator);
    let handler_content = {
        let mut s = String::from("__RAW__() => _value => (_ctx.");
        s.push_str(binding.as_str());
        s.push_str(" = _value)");
        s
    };
    let handler_node = SimpleExpressionNode::new(handler_content, true, SourceLocation::STUB);
    let mut handler_values = Vec::new_in(ctx.allocator);
    handler_values.push(Box::new_in(handler_node, ctx.allocator));
    props.push(IRProp {
        key: event_key_box,
        values: handler_values,
        is_component: true,
    });

    if !dir.modifiers.is_empty() {
        let mod_key_name = if prop_name == "modelValue" {
            String::from("modelModifiers")
        } else {
            let mut s = prop_name.clone();
            s.push_str("Modifiers");
            s
        };
        let mod_key_node = SimpleExpressionNode::new(mod_key_name, true, SourceLocation::STUB);
        let mod_key = Box::new_in(mod_key_node, ctx.allocator);
        let mut mod_content = String::from("__RAW__() => ({ ");
        for (i, m) in dir.modifiers.iter().enumerate() {
            if i > 0 {
                mod_content.push_str(", ");
            }
            mod_content.push_str(m.content.as_str());
            mod_content.push_str(": true");
        }
        mod_content.push_str(" })");
        let mod_val_node = SimpleExpressionNode::new(mod_content, true, SourceLocation::STUB);
        let mut mod_values = Vec::new_in(ctx.allocator);
        mod_values.push(Box::new_in(mod_val_node, ctx.allocator));
        props.push(IRProp {
            key: mod_key,
            values: mod_values,
            is_component: true,
        });
    }
}
