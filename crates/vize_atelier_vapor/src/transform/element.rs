//! Element transformation.
//!
//! Handles `ElementNode` transformation dispatch. Template construction,
//! deferred child wiring, and component lowering live in focused submodules so
//! each transform path stays independently reviewable.

mod component;
mod deferred;
mod template;

use vize_carton::{Box, String, Vec, append, cstr};

use crate::ir::{
    BlockIRNode, ChildRefIRNode, ComponentKind, CreateComponentIRNode, IRProp, IRSlot,
    NextRefIRNode, OperationNode, SetTemplateRefIRNode, SlotOutletIRNode,
};
use vize_atelier_core::{
    ElementNode, ElementType, ExpressionNode, PropNode, SimpleExpressionNode, SourceLocation,
    TemplateChildNode,
};

use self::{
    component::transform_component,
    deferred::{
        transform_element_with_control_flow_children, transform_element_with_dynamic_children,
    },
    template::{generate_element_template, is_static_element, transform_template_ref},
};

use super::{
    context::TransformContext,
    control::{
        transform_for_node, transform_for_node_deferred_parent, transform_for_node_into_parent,
        transform_if_node, transform_if_node_deferred_parent, transform_if_node_into_parent,
    },
    directive::transform_directive,
    text::{transform_interpolation, transform_text, transform_text_children},
    transform_children,
};

/// Lower an element-like AST node into Vapor IR operations.
pub(crate) fn transform_element<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    block: &mut BlockIRNode<'a>,
) {
    let non_reactive = classify_non_reactive_directive(el);
    if let Some(ref memo_error) = non_reactive.memo_error {
        ctx.push_diagnostic(memo_error.clone());
    }
    let entered_non_reactive = non_reactive.should_lower_as_once;
    if entered_non_reactive {
        ctx.enter_non_reactive_scope();
    }

    // Template elements don't consume an ID - they just wrap children
    if el.tag_type == ElementType::Template {
        for child in el.children.iter() {
            match child {
                TemplateChildNode::Element(child_el) => {
                    transform_element(ctx, child_el, block);
                }
                TemplateChildNode::Text(text) => {
                    transform_text(ctx, text, block);
                }
                TemplateChildNode::Interpolation(interp) => {
                    transform_interpolation(ctx, interp, block);
                }
                TemplateChildNode::If(if_node) => {
                    transform_if_node(ctx, if_node, block);
                }
                TemplateChildNode::For(for_node) => {
                    transform_for_node(ctx, for_node, block);
                }
                _ => {}
            }
        }
        if entered_non_reactive {
            ctx.exit_non_reactive_scope();
        }
        return;
    }

    // Check if this element has non-static children that require
    // deferred ID allocation (so inner templates/IDs come first).
    let has_control_flow_children = el.tag_type == ElementType::Element
        && el
            .children
            .iter()
            .any(|c| matches!(c, TemplateChildNode::If(_) | TemplateChildNode::For(_)));
    let has_dynamic_element_children = el.tag_type == ElementType::Element
        && !has_control_flow_children
        && el.children.iter().any(
            |c| matches!(c, TemplateChildNode::Element(child_el) if !is_static_element(child_el)),
        );

    if has_dynamic_element_children {
        // Dynamic element children: allocate child IDs first, then parent ID.
        // Use child/next navigation instead of separate templates.
        transform_element_with_dynamic_children(ctx, el, block);
        if entered_non_reactive {
            ctx.exit_non_reactive_scope();
        }
        return;
    }

    if has_control_flow_children {
        // Control flow children (v-if/v-for): defer parent ID and template
        // allocation until after children, so inner IDs/templates come first.
        transform_element_with_control_flow_children(ctx, el, block);
        if entered_non_reactive {
            ctx.exit_non_reactive_scope();
        }
        return;
    }

    // Components handle their own ID allocation (slots consume IDs before the component)
    // Also handle <component :is="..."> (dynamic component) which the parser classifies as Element
    if el.tag_type == ElementType::Component || el.tag.as_str() == "component" {
        transform_component(ctx, el, block, None, None, None, true);
        if entered_non_reactive {
            ctx.exit_non_reactive_scope();
        }
        return;
    }

    let element_id = ctx.next_id();

    match el.tag_type {
        ElementType::Element => {
            let template = generate_element_template(el);

            // Process props and events
            for prop in el.props.iter() {
                match prop {
                    PropNode::Directive(dir) => {
                        transform_directive(ctx, dir, element_id, el, block);
                    }
                    PropNode::Attribute(_attr) => {
                        // Static attributes are included in the template
                    }
                }
            }

            transform_template_ref(ctx, el, element_id, block);

            // Check if we have mixed text and interpolation children
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
                // Collect all text parts and interpolations together
                transform_text_children(ctx, &el.children, element_id, block);
            }

            // Register template (no deferred children to process)
            ctx.add_template(element_id, template);
        }
        ElementType::Component => {
            let mut props = Vec::new_in(ctx.allocator);
            let slots = Vec::new_in(ctx.allocator);

            // Process props (v-bind and v-on directives, and static attributes)
            for prop in el.props.iter() {
                match prop {
                    PropNode::Directive(dir) => {
                        if dir.name.as_str() == "bind" {
                            // v-bind -> prop, v-bind="obj" -> ordered spread source
                            if let Some(ref arg) = dir.arg {
                                if let ExpressionNode::Simple(key_exp) = arg {
                                    let key_node = SimpleExpressionNode::new(
                                        key_exp.content.clone(),
                                        key_exp.is_static,
                                        key_exp.loc.clone(),
                                    );
                                    let key = Box::new_in(key_node, ctx.allocator);

                                    let mut values = Vec::new_in(ctx.allocator);
                                    if let Some(ref exp) = dir.exp
                                        && let ExpressionNode::Simple(val_exp) = exp
                                    {
                                        let val_node = SimpleExpressionNode::new(
                                            val_exp.content.clone(),
                                            val_exp.is_static,
                                            val_exp.loc.clone(),
                                        );
                                        values.push(Box::new_in(val_node, ctx.allocator));
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
                        } else if dir.name.as_str() == "on" {
                            // v-on -> onXxx prop
                            if let Some(ref arg) = dir.arg
                                && let ExpressionNode::Simple(event_exp) = arg
                            {
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
                                if let Some(ref exp) = dir.exp
                                    && let ExpressionNode::Simple(val_exp) = exp
                                {
                                    let val_node = SimpleExpressionNode::new(
                                        val_exp.content.clone(),
                                        val_exp.is_static,
                                        val_exp.loc.clone(),
                                    );
                                    values.push(Box::new_in(val_node, ctx.allocator));
                                }

                                props.push(IRProp {
                                    key,
                                    values,
                                    is_component: true,
                                });
                            }
                        } else if dir.name.as_str() == "model" {
                            // v-model -> modelValue + onUpdate:modelValue props
                            let binding = if let Some(ref exp) = dir.exp {
                                match exp {
                                    ExpressionNode::Simple(s) => s.content.clone(),
                                    _ => String::from(""),
                                }
                            } else {
                                String::from("")
                            };

                            // Determine prop name from argument (default: "modelValue")
                            let prop_name = dir
                                .arg
                                .as_ref()
                                .map(|arg| match arg {
                                    ExpressionNode::Simple(s) => s.content.clone(),
                                    _ => String::from("modelValue"),
                                })
                                .unwrap_or_else(|| String::from("modelValue"));

                            // Add modelValue prop
                            let key_node = SimpleExpressionNode::new(
                                prop_name.clone(),
                                true,
                                SourceLocation::STUB,
                            );
                            let key = Box::new_in(key_node, ctx.allocator);
                            let mut values = Vec::new_in(ctx.allocator);
                            let val_node = SimpleExpressionNode::new(
                                binding.clone(),
                                false,
                                SourceLocation::STUB,
                            );
                            values.push(Box::new_in(val_node, ctx.allocator));
                            props.push(IRProp {
                                key,
                                values,
                                is_component: true,
                            });

                            // Add onUpdate:propName event prop
                            let event_key = {
                                let mut s = String::from("onUpdate:");
                                s.push_str(prop_name.as_str());
                                s
                            };
                            let event_key_node =
                                SimpleExpressionNode::new(event_key, true, SourceLocation::STUB);
                            let event_key_box = Box::new_in(event_key_node, ctx.allocator);
                            // Handler getter: the Vapor runtime resolves raw
                            // component props lazily before emit invokes it.
                            let handler_content = {
                                let mut s = String::from("__RAW__() => _value => (_ctx.");
                                s.push_str(binding.as_str());
                                s.push_str(" = _value)");
                                s
                            };
                            let handler_node = SimpleExpressionNode::new(
                                handler_content,
                                true,
                                SourceLocation::STUB,
                            );
                            let mut handler_values = Vec::new_in(ctx.allocator);
                            handler_values.push(Box::new_in(handler_node, ctx.allocator));
                            props.push(IRProp {
                                key: event_key_box,
                                values: handler_values,
                                is_component: true,
                            });

                            // Add modifiers prop if present
                            if !dir.modifiers.is_empty() {
                                let mod_key_name = if prop_name == "modelValue" {
                                    String::from("modelModifiers")
                                } else {
                                    let mut s = prop_name.clone();
                                    s.push_str("Modifiers");
                                    s
                                };
                                let mod_key_node = SimpleExpressionNode::new(
                                    mod_key_name,
                                    true,
                                    SourceLocation::STUB,
                                );
                                let mod_key = Box::new_in(mod_key_node, ctx.allocator);
                                // Build modifiers object content
                                let mut mod_content = String::from("__RAW__() => ({ ");
                                for (i, m) in dir.modifiers.iter().enumerate() {
                                    if i > 0 {
                                        mod_content.push_str(", ");
                                    }
                                    mod_content.push_str(m.content.as_str());
                                    mod_content.push_str(": true");
                                }
                                mod_content.push_str(" })");
                                let mod_val_node = SimpleExpressionNode::new(
                                    mod_content,
                                    true,
                                    SourceLocation::STUB,
                                );
                                let mut mod_values = Vec::new_in(ctx.allocator);
                                mod_values.push(Box::new_in(mod_val_node, ctx.allocator));
                                props.push(IRProp {
                                    key: mod_key,
                                    values: mod_values,
                                    is_component: true,
                                });
                            }
                        }
                    }
                    PropNode::Attribute(attr) => {
                        // Static attribute -> prop
                        let key_node = SimpleExpressionNode::new(
                            attr.name.clone(),
                            true,
                            SourceLocation::STUB,
                        );
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

            let create_component = CreateComponentIRNode {
                id: element_id,
                tag: el.tag.clone(),
                props,
                slots,
                asset: true,
                once: false,
                dynamic_slots: false,
                kind: crate::ir::ComponentKind::Regular,
                is_expr: None,
                v_show: None,
                parent: None,
                anchor: None,
            };

            block
                .operation
                .push(OperationNode::CreateComponent(create_component));
        }
        ElementType::Slot => {
            let name = get_slot_outlet_name(ctx, el);
            let props = get_slot_outlet_props(ctx, el);
            let fallback = (!el.children.is_empty()).then(|| transform_children(ctx, &el.children));
            let slot_outlet = SlotOutletIRNode {
                id: element_id,
                name,
                props,
                fallback,
            };

            block.operation.push(OperationNode::SlotOutlet(slot_outlet));
        }
        ElementType::Template => {
            // Panic path by transform invariant: `transform_element` peels
            // template nodes at function entry because they do not create DOM
            // elements. Reaching this arm means that guard was bypassed in a
            // future edit, so continuing would emit an invalid Vapor operation.
            unreachable!("Template elements handled at top of transform_element");
        }
    }

    block.returns.push(element_id);

    if entered_non_reactive {
        ctx.exit_non_reactive_scope();
    }
}

struct NonReactiveDirective {
    should_lower_as_once: bool,
    memo_error: Option<String>,
}

fn classify_non_reactive_directive(el: &ElementNode<'_>) -> NonReactiveDirective {
    let has_once = el
        .props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(dir) if dir.name.as_str() == "once"));
    if has_once {
        return NonReactiveDirective {
            should_lower_as_once: true,
            memo_error: None,
        };
    }

    for prop in el.props.iter() {
        let PropNode::Directive(dir) = prop else {
            continue;
        };
        if dir.name.as_str() != "memo" {
            continue;
        }

        let Some(ExpressionNode::Simple(exp)) = dir.exp.as_ref() else {
            return NonReactiveDirective {
                should_lower_as_once: false,
                memo_error: Some(vize_carton::String::from(
                    "v-memo is not supported in Vapor yet. Use v-once or v-memo=\"[]\" until memo guards are implemented.",
                )),
            };
        };

        if exp.content.trim() == "[]" {
            return NonReactiveDirective {
                should_lower_as_once: true,
                memo_error: None,
            };
        }

        return NonReactiveDirective {
            should_lower_as_once: false,
            memo_error: Some(vize_carton::String::from(
                "v-memo with dependencies is not supported in Vapor yet. Use v-once or v-memo=\"[]\" until memo guards are implemented.",
            )),
        };
    }

    NonReactiveDirective {
        should_lower_as_once: false,
        memo_error: None,
    }
}

fn get_slot_outlet_name<'a>(
    ctx: &TransformContext<'a>,
    el: &ElementNode<'a>,
) -> Box<'a, SimpleExpressionNode<'a>> {
    for prop in el.props.iter() {
        match prop {
            PropNode::Attribute(attr) => {
                if attr.name == "name"
                    && let Some(ref value) = attr.value
                {
                    return Box::new_in(
                        SimpleExpressionNode::new(
                            value.content.clone(),
                            true,
                            SourceLocation::STUB,
                        ),
                        ctx.allocator,
                    );
                }
            }
            PropNode::Directive(dir) => {
                if dir.name == "bind"
                    && let Some(ExpressionNode::Simple(arg)) = dir.arg.as_ref()
                    && arg.content == "name"
                    && let Some(ExpressionNode::Simple(exp)) = dir.exp.as_ref()
                {
                    return Box::new_in(
                        SimpleExpressionNode::new(
                            exp.content.clone(),
                            exp.is_static,
                            exp.loc.clone(),
                        ),
                        ctx.allocator,
                    );
                }
            }
        }
    }

    Box::new_in(
        SimpleExpressionNode::new("default", true, SourceLocation::STUB),
        ctx.allocator,
    )
}

fn get_slot_outlet_props<'a>(
    ctx: &TransformContext<'a>,
    el: &ElementNode<'a>,
) -> Vec<'a, IRProp<'a>> {
    let mut props = Vec::new_in(ctx.allocator);

    for prop in el.props.iter() {
        match prop {
            PropNode::Attribute(attr) => {
                if attr.name == "name" {
                    continue;
                }

                let key = Box::new_in(
                    SimpleExpressionNode::new(attr.name.clone(), true, SourceLocation::STUB),
                    ctx.allocator,
                );
                let mut values = Vec::new_in(ctx.allocator);
                if let Some(ref value) = attr.value {
                    values.push(Box::new_in(
                        SimpleExpressionNode::new(
                            value.content.clone(),
                            true,
                            SourceLocation::STUB,
                        ),
                        ctx.allocator,
                    ));
                }

                props.push(IRProp {
                    key,
                    values,
                    is_component: false,
                });
            }
            PropNode::Directive(dir) => {
                if dir.name != "bind" {
                    continue;
                }

                match (dir.arg.as_ref(), dir.exp.as_ref()) {
                    (Some(ExpressionNode::Simple(arg)), Some(ExpressionNode::Simple(exp))) => {
                        if arg.content == "name" {
                            continue;
                        }

                        let key = Box::new_in(
                            SimpleExpressionNode::new(
                                arg.content.clone(),
                                arg.is_static,
                                arg.loc.clone(),
                            ),
                            ctx.allocator,
                        );
                        let mut values = Vec::new_in(ctx.allocator);
                        values.push(Box::new_in(
                            SimpleExpressionNode::new(
                                exp.content.clone(),
                                exp.is_static,
                                exp.loc.clone(),
                            ),
                            ctx.allocator,
                        ));

                        props.push(IRProp {
                            key,
                            values,
                            is_component: false,
                        });
                    }
                    (None, Some(ExpressionNode::Simple(exp))) => {
                        let key = Box::new_in(
                            SimpleExpressionNode::new("$", true, SourceLocation::STUB),
                            ctx.allocator,
                        );
                        let mut values = Vec::new_in(ctx.allocator);
                        values.push(Box::new_in(
                            SimpleExpressionNode::new(
                                exp.content.clone(),
                                exp.is_static,
                                exp.loc.clone(),
                            ),
                            ctx.allocator,
                        ));

                        props.push(IRProp {
                            key,
                            values,
                            is_component: false,
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    props
}
