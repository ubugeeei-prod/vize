//! Directive transformation.
//!
//! Handles v-bind, v-on, v-if, v-for, v-html, v-text, and custom directives.

use vize_carton::{Box, Vec};

use crate::ir::{
    BlockIRNode, DirectiveIRNode, ForIRNode, IREffect, IRProp, IfIRNode, OperationNode,
    SetEventIRNode, SetHtmlIRNode, SetPropIRNode, SetTextIRNode,
};
use vize_atelier_core::{
    DirectiveNode, ElementNode, ElementType, ExpressionNode, PropNode, SimpleExpressionNode,
    SourceLocation,
};

use super::{context::TransformContext, transform_children};

/// Transform directive
pub(crate) fn transform_directive<'a>(
    ctx: &mut TransformContext<'a>,
    dir: &DirectiveNode<'a>,
    element_id: usize,
    el: &ElementNode<'a>,
    block: &mut BlockIRNode<'a>,
) {
    match dir.name.as_str() {
        "bind" => {
            // Skip :key - handled by v-for key function
            if let Some(ref arg) = dir.arg {
                if let ExpressionNode::Simple(key_exp) = arg {
                    if key_exp.content.as_str() == "key" {
                        return;
                    }
                }
            }

            // Check modifiers
            let has_camel = dir.modifiers.iter().any(|m| m.content.as_str() == "camel");
            let has_prop = dir.modifiers.iter().any(|m| m.content.as_str() == "prop");

            if let Some(ref arg) = dir.arg {
                if let ExpressionNode::Simple(key_exp) = arg {
                    if el.tag_type == ElementType::Element
                        && matches!(key_exp.content.as_str(), "ref" | "ref_for" | "ref_key")
                    {
                        return;
                    }

                    // Dynamic attribute name (e.g. :[attr]="value") -> SetDynamicProps
                    if !key_exp.is_static {
                        if let Some(ref exp) = dir.exp {
                            if let ExpressionNode::Simple(val_exp) = exp {
                                let mut props = Vec::new_in(ctx.allocator);
                                // Create an expression that represents { [key]: value }
                                let obj_content = {
                                    let mut s = vize_carton::String::from("{ [");
                                    s.push_str(key_exp.content.as_str());
                                    s.push_str("]: ");
                                    s.push_str(val_exp.content.as_str());
                                    s.push_str(" }");
                                    s
                                };
                                let obj_node = SimpleExpressionNode::new(
                                    obj_content,
                                    false,
                                    key_exp.loc.clone(),
                                );
                                props.push(Box::new_in(obj_node, ctx.allocator));

                                let set_dynamic = crate::ir::SetDynamicPropsIRNode {
                                    element: element_id,
                                    props,
                                    is_event: false,
                                };
                                let mut effect_ops = Vec::new_in(ctx.allocator);
                                effect_ops.push(OperationNode::SetDynamicProps(set_dynamic));
                                block.effect.push(IREffect {
                                    operations: effect_ops,
                                });
                            }
                        }
                        return;
                    }

                    // Apply .camel modifier: camelize the key
                    let key_content = if has_camel {
                        camelize(&key_exp.content)
                    } else {
                        key_exp.content.clone()
                    };

                    let key_node = SimpleExpressionNode::new(
                        key_content,
                        key_exp.is_static,
                        key_exp.loc.clone(),
                    );
                    let key = Box::new_in(key_node, ctx.allocator);

                    let values = if let Some(ref exp) = dir.exp {
                        if let ExpressionNode::Simple(val_exp) = exp {
                            let mut v = Vec::new_in(ctx.allocator);
                            let val_node = SimpleExpressionNode::new(
                                val_exp.content.clone(),
                                val_exp.is_static,
                                val_exp.loc.clone(),
                            );
                            v.push(Box::new_in(val_node, ctx.allocator));
                            v
                        } else {
                            Vec::new_in(ctx.allocator)
                        }
                    } else {
                        Vec::new_in(ctx.allocator)
                    };

                    // Check for static class attribute to merge
                    let final_values = if key_exp.content.as_str() == "class" {
                        merge_static_class(ctx, el, values)
                    } else {
                        values
                    };

                    let set_prop = SetPropIRNode {
                        element: element_id,
                        prop: IRProp {
                            key,
                            values: final_values,
                            is_component: el.tag_type == ElementType::Component,
                        },
                        tag: el.tag.clone(),
                        camel: has_camel,
                        prop_modifier: has_prop,
                    };

                    // Reactive prop - add to effects
                    let mut effect_ops = Vec::new_in(ctx.allocator);
                    effect_ops.push(OperationNode::SetProp(set_prop));
                    block.effect.push(IREffect {
                        operations: effect_ops,
                    });
                }
            } else {
                // v-bind without arg = v-bind object (v-bind="attrs")
                if let Some(ref exp) = dir.exp {
                    if let ExpressionNode::Simple(val_exp) = exp {
                        let mut props = Vec::new_in(ctx.allocator);
                        let val_node = SimpleExpressionNode::new(
                            val_exp.content.clone(),
                            val_exp.is_static,
                            val_exp.loc.clone(),
                        );
                        props.push(Box::new_in(val_node, ctx.allocator));

                        let set_dynamic = crate::ir::SetDynamicPropsIRNode {
                            element: element_id,
                            props,
                            is_event: false,
                        };
                        let mut effect_ops = Vec::new_in(ctx.allocator);
                        effect_ops.push(OperationNode::SetDynamicProps(set_dynamic));
                        block.effect.push(IREffect {
                            operations: effect_ops,
                        });
                    }
                }
            }
        }
        "on" => {
            if let Some(ref arg) = dir.arg {
                if let ExpressionNode::Simple(key_exp) = arg {
                    let key_node = SimpleExpressionNode::new(
                        key_exp.content.clone(),
                        key_exp.is_static,
                        key_exp.loc.clone(),
                    );
                    let key = Box::new_in(key_node, ctx.allocator);

                    let value = if let Some(ref exp) = dir.exp {
                        if let ExpressionNode::Simple(val_exp) = exp {
                            let val_node = SimpleExpressionNode::new(
                                val_exp.content.clone(),
                                val_exp.is_static,
                                val_exp.loc.clone(),
                            );
                            Some(Box::new_in(val_node, ctx.allocator))
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    // Parse modifiers
                    let mut modifiers = crate::ir::EventModifiers::default();
                    let event_name = key_exp.content.as_str();
                    let is_dynamic = !key_exp.is_static;

                    for m in dir.modifiers.iter() {
                        match m.content.as_str() {
                            "once" => modifiers.options.once = true,
                            "capture" => modifiers.options.capture = true,
                            "passive" => modifiers.options.passive = true,
                            "stop" | "prevent" | "self" => {
                                modifiers.non_keys.push(m.content.clone());
                            }
                            "enter" | "tab" | "delete" | "esc" | "space" | "up" | "down"
                            | "left" | "right" => {
                                modifiers.keys.push(m.content.clone());
                            }
                            _ => {
                                modifiers.non_keys.push(m.content.clone());
                            }
                        }
                    }

                    // Determine delegation
                    let delegate = !is_dynamic
                        && !modifiers.options.once
                        && !modifiers.options.capture
                        && !modifiers.options.passive
                        && is_delegatable_event(event_name);

                    let set_event = SetEventIRNode {
                        element: element_id,
                        key,
                        value,
                        modifiers,
                        delegate,
                        effect: is_dynamic,
                    };

                    block.operation.push(OperationNode::SetEvent(set_event));
                }
            } else {
                // v-on without arg = v-on object (v-on="handlers")
                if let Some(ref exp) = dir.exp {
                    if let ExpressionNode::Simple(val_exp) = exp {
                        let mut values = Vec::new_in(ctx.allocator);
                        let val_node = SimpleExpressionNode::new(
                            val_exp.content.clone(),
                            val_exp.is_static,
                            val_exp.loc.clone(),
                        );
                        values.push(Box::new_in(val_node, ctx.allocator));

                        let set_dynamic = crate::ir::SetDynamicPropsIRNode {
                            element: element_id,
                            props: values,
                            is_event: true,
                        };
                        let mut effect_ops = Vec::new_in(ctx.allocator);
                        effect_ops.push(OperationNode::SetDynamicProps(set_dynamic));
                        block.effect.push(IREffect {
                            operations: effect_ops,
                        });
                    }
                }
            }
        }
        "if" => {
            // v-if
            if let Some(ref exp) = dir.exp {
                if let ExpressionNode::Simple(cond_exp) = exp {
                    let cond_node = SimpleExpressionNode::new(
                        cond_exp.content.clone(),
                        cond_exp.is_static,
                        cond_exp.loc.clone(),
                    );
                    let condition = Box::new_in(cond_node, ctx.allocator);
                    let positive = transform_children(ctx, &el.children);

                    let if_node = IfIRNode {
                        id: ctx.next_id(),
                        condition,
                        positive,
                        negative: None,
                        once: false,
                        parent: None,
                        anchor: None,
                    };

                    block
                        .operation
                        .push(OperationNode::If(Box::new_in(if_node, ctx.allocator)));
                }
            }
        }
        "for" => {
            // v-for
            if let Some(ref exp) = dir.exp {
                if let ExpressionNode::Simple(source_exp) = exp {
                    let source_node = SimpleExpressionNode::new(
                        source_exp.content.clone(),
                        source_exp.is_static,
                        source_exp.loc.clone(),
                    );
                    let source = Box::new_in(source_node, ctx.allocator);
                    let render = transform_children(ctx, &el.children);

                    let for_node = ForIRNode {
                        id: ctx.next_id(),
                        source,
                        value: None,
                        key: None,
                        index: None,
                        key_prop: None,
                        render,
                        once: false,
                        component: el.tag_type == ElementType::Component,
                        only_child: false,
                        parent: None,
                        anchor: None,
                    };

                    block
                        .operation
                        .push(OperationNode::For(Box::new_in(for_node, ctx.allocator)));
                }
            }
        }
        "html" => {
            // v-html
            if let Some(ref exp) = dir.exp {
                if let ExpressionNode::Simple(val_exp) = exp {
                    let val_node = SimpleExpressionNode::new(
                        val_exp.content.clone(),
                        val_exp.is_static,
                        val_exp.loc.clone(),
                    );
                    let value = Box::new_in(val_node, ctx.allocator);
                    let set_html = SetHtmlIRNode {
                        element: element_id,
                        value,
                    };

                    let mut effect_ops = Vec::new_in(ctx.allocator);
                    effect_ops.push(OperationNode::SetHtml(set_html));
                    block.effect.push(IREffect {
                        operations: effect_ops,
                    });
                }
            }
        }
        "text" => {
            // v-text
            if let Some(ref exp) = dir.exp {
                if let ExpressionNode::Simple(val_exp) = exp {
                    let mut values = Vec::new_in(ctx.allocator);
                    let val_node = SimpleExpressionNode::new(
                        val_exp.content.clone(),
                        val_exp.is_static,
                        val_exp.loc.clone(),
                    );
                    values.push(Box::new_in(val_node, ctx.allocator));

                    let set_text = SetTextIRNode {
                        element: element_id,
                        values,
                    };

                    let mut effect_ops = Vec::new_in(ctx.allocator);
                    effect_ops.push(OperationNode::SetText(set_text));
                    block.effect.push(IREffect {
                        operations: effect_ops,
                    });
                }
            }
        }
        "show" => {
            // v-show - builtin directive
            let mut new_dir = DirectiveNode::new(ctx.allocator, dir.name.clone(), dir.loc.clone());
            if let Some(ref exp) = dir.exp {
                if let ExpressionNode::Simple(s) = exp {
                    let node =
                        SimpleExpressionNode::new(s.content.clone(), s.is_static, s.loc.clone());
                    new_dir.exp = Some(ExpressionNode::Simple(Box::new_in(node, ctx.allocator)));
                }
            }

            let dir_node = DirectiveIRNode {
                element: element_id,
                dir: Box::new_in(new_dir, ctx.allocator),
                name: vize_carton::String::from("vShow"),
                builtin: true,
                tag: el.tag.clone(),
                input_type: get_static_attr(el, "type"),
            };

            block.operation.push(OperationNode::Directive(dir_node));
        }
        "model" => {
            // v-model - builtin directive
            let mut new_dir = DirectiveNode::new(ctx.allocator, dir.name.clone(), dir.loc.clone());
            if let Some(ref exp) = dir.exp {
                if let ExpressionNode::Simple(s) = exp {
                    let node =
                        SimpleExpressionNode::new(s.content.clone(), s.is_static, s.loc.clone());
                    new_dir.exp = Some(ExpressionNode::Simple(Box::new_in(node, ctx.allocator)));
                }
            }
            if let Some(ref arg) = dir.arg {
                if let ExpressionNode::Simple(s) = arg {
                    let node =
                        SimpleExpressionNode::new(s.content.clone(), s.is_static, s.loc.clone());
                    new_dir.arg = Some(ExpressionNode::Simple(Box::new_in(node, ctx.allocator)));
                }
            }
            for m in dir.modifiers.iter() {
                new_dir.modifiers.push(SimpleExpressionNode::new(
                    m.content.clone(),
                    m.is_static,
                    m.loc.clone(),
                ));
            }

            let dir_node = DirectiveIRNode {
                element: element_id,
                dir: Box::new_in(new_dir, ctx.allocator),
                name: vize_carton::String::from("model"),
                builtin: true,
                tag: el.tag.clone(),
                input_type: get_static_attr(el, "type"),
            };

            block.operation.push(OperationNode::Directive(dir_node));
        }
        _ => {
            // Custom directive - create a copy of the directive
            let new_dir = DirectiveNode::new(ctx.allocator, dir.name.clone(), dir.loc.clone());

            let dir_node = DirectiveIRNode {
                element: element_id,
                dir: Box::new_in(new_dir, ctx.allocator),
                name: dir.name.clone(),
                builtin: false,
                tag: el.tag.clone(),
                input_type: vize_carton::String::from(""),
            };

            block.operation.push(OperationNode::Directive(dir_node));
        }
    }
}

/// Check if an event can use delegation
fn is_delegatable_event(name: &str) -> bool {
    matches!(
        name,
        "click"
            | "dblclick"
            | "mousedown"
            | "mouseup"
            | "mousemove"
            | "mouseenter"
            | "mouseleave"
            | "mouseover"
            | "mouseout"
            | "keydown"
            | "keyup"
            | "keypress"
            | "pointerdown"
            | "pointerup"
            | "pointermove"
            | "pointerenter"
            | "pointerleave"
            | "pointerover"
            | "pointerout"
            | "touchstart"
            | "touchend"
            | "touchmove"
            | "focusin"
            | "focusout"
            | "input"
            | "change"
            | "contextmenu"
            | "wheel"
            | "scroll"
            | "drag"
            | "dragstart"
            | "dragend"
            | "dragenter"
            | "dragleave"
            | "dragover"
            | "drop"
    )
}

/// Camelize a hyphenated string (e.g. "view-box" -> "viewBox")
fn camelize(s: &str) -> vize_carton::String {
    let mut result = vize_carton::String::default();
    let mut capitalize_next = false;
    for c in s.chars() {
        if c == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

/// Merge static class attribute value into the dynamic class values
fn merge_static_class<'a>(
    ctx: &mut TransformContext<'a>,
    el: &ElementNode<'a>,
    dynamic_values: Vec<'a, Box<'a, SimpleExpressionNode<'a>>>,
) -> Vec<'a, Box<'a, SimpleExpressionNode<'a>>> {
    // Look for a static class="..." attribute
    let static_class = el.props.iter().find_map(|p| {
        if let PropNode::Attribute(attr) = p {
            if attr.name.as_str() == "class" {
                if let Some(ref value) = attr.value {
                    return Some(value.content.clone());
                }
            }
        }
        None
    });

    if let Some(static_val) = static_class {
        // Create a merged values list: the static class as the first entry
        let mut merged = Vec::new_in(ctx.allocator);
        let static_node = SimpleExpressionNode::new(static_val, true, SourceLocation::STUB);
        merged.push(Box::new_in(static_node, ctx.allocator));
        for v in dynamic_values.into_iter() {
            merged.push(v);
        }
        merged
    } else {
        dynamic_values
    }
}

/// Get a static attribute value from an element
fn get_static_attr(el: &ElementNode<'_>, attr_name: &str) -> vize_carton::String {
    for prop in el.props.iter() {
        if let PropNode::Attribute(attr) = prop {
            if attr.name.as_str() == attr_name {
                if let Some(ref value) = attr.value {
                    return value.content.clone();
                }
            }
        }
    }
    vize_carton::String::from("")
}
