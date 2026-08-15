//! Directive transformation.
//!
//! Handles v-bind, v-on, v-if, v-for, v-html, v-text, and custom directives.

use vize_carton::{Box, Vec};

use crate::ir::{
    BlockIRNode, DirectiveIRNode, ForIRNode, IRProp, IfIRNode, OperationNode, SetEventIRNode,
    SetHtmlIRNode, SetPropIRNode, SetTextIRNode,
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
    match dir.name {
        "bind" => {
            // Skip :key - handled by v-for key function
            if let Some(ref arg) = dir.arg
                && let ExpressionNode::Simple(key_exp) = arg
                && key_exp.content == "key"
            {
                return;
            }

            // Check modifiers
            let has_camel = dir.modifiers.iter().any(|m| m.content == "camel");
            let has_prop = dir.modifiers.iter().any(|m| m.content == "prop");

            if let Some(ref arg) = dir.arg {
                if let ExpressionNode::Simple(key_exp) = arg {
                    if el.tag_type == ElementType::Element
                        && matches!(key_exp.content, "ref" | "ref_for" | "ref_key")
                    {
                        return;
                    }

                    // Dynamic attribute name (e.g. :[attr]="value") -> SetDynamicProps
                    if !key_exp.is_static {
                        if let Some(ref exp) = dir.exp
                            && let ExpressionNode::Simple(val_exp) = exp
                        {
                            let mut props = Vec::new_in(&ctx.allocator);
                            // Create an expression that represents { [key]: value }
                            let obj_content = {
                                let mut s = vize_carton::String::from("{ [");
                                s.push_str(key_exp.content);
                                s.push_str("]: ");
                                s.push_str(val_exp.content);
                                s.push_str(" }");
                                s
                            };
                            let obj = ctx.allocator.alloc_str(&obj_content);
                            let node = SimpleExpressionNode::new(obj, false, key_exp.loc.clone());
                            props.push(Box::new_in(node, &ctx.allocator));

                            let set_dynamic = crate::ir::SetDynamicPropsIRNode {
                                element: element_id,
                                props,
                                is_event: false,
                            };
                            ctx.push_dynamic_operation(
                                block,
                                OperationNode::SetDynamicProps(set_dynamic),
                            );
                        }
                        return;
                    }

                    // Apply .camel modifier: camelize the key
                    let key_content = if has_camel {
                        ctx.interner.intern(&camelize(key_exp.content))
                    } else {
                        key_exp.content
                    };

                    let key_node = SimpleExpressionNode::new(
                        key_content,
                        key_exp.is_static,
                        key_exp.loc.clone(),
                    );
                    let key = Box::new_in(key_node, &ctx.allocator);

                    let values = if let Some(ref exp) = dir.exp {
                        if let ExpressionNode::Simple(val_exp) = exp {
                            let mut v = Vec::new_in(&ctx.allocator);
                            let val_node = SimpleExpressionNode::from_node(val_exp);
                            v.push(Box::new_in(val_node, &ctx.allocator));
                            v
                        } else {
                            Vec::new_in(&ctx.allocator)
                        }
                    } else {
                        Vec::new_in(&ctx.allocator)
                    };

                    // Check for static class attribute to merge
                    let final_values = if key_exp.content == "class" {
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
                        tag: el.tag,
                        camel: has_camel,
                        prop_modifier: has_prop,
                    };

                    // Reactive prop - add to effects
                    ctx.push_dynamic_operation(block, OperationNode::SetProp(set_prop));
                }
            } else {
                // v-bind without arg = v-bind object (v-bind="attrs")
                if let Some(ref exp) = dir.exp
                    && let ExpressionNode::Simple(val_exp) = exp
                {
                    let mut props = Vec::new_in(&ctx.allocator);
                    let val_node = SimpleExpressionNode::from_node(val_exp);
                    props.push(Box::new_in(val_node, &ctx.allocator));

                    let set_dynamic = crate::ir::SetDynamicPropsIRNode {
                        element: element_id,
                        props,
                        is_event: false,
                    };
                    ctx.push_dynamic_operation(block, OperationNode::SetDynamicProps(set_dynamic));
                }
            }
        }
        "on" => {
            if let Some(ref arg) = dir.arg {
                if let ExpressionNode::Simple(key_exp) = arg {
                    let key_node = SimpleExpressionNode::from_node(key_exp);
                    let key = Box::new_in(key_node, &ctx.allocator);

                    let value = if let Some(ref exp) = dir.exp {
                        if let ExpressionNode::Simple(val_exp) = exp {
                            let val_node = SimpleExpressionNode::from_node(val_exp);
                            Some(Box::new_in(val_node, &ctx.allocator))
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    // Parse modifiers
                    let mut modifiers = crate::ir::EventModifiers::new(ctx.allocator);
                    let event_name = key_exp.content;
                    let is_dynamic = !key_exp.is_static;

                    for m in dir.modifiers.iter() {
                        match m.content {
                            "once" => modifiers.options.once = true,
                            "capture" => modifiers.options.capture = true,
                            "passive" => modifiers.options.passive = true,
                            "stop" | "prevent" | "self" => {
                                modifiers.non_keys.push(m.content);
                            }
                            "enter" | "tab" | "delete" | "esc" | "space" | "up" | "down"
                            | "left" | "right" => {
                                modifiers.keys.push(m.content);
                            }
                            _ => {
                                modifiers.non_keys.push(m.content);
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
                if let Some(ref exp) = dir.exp
                    && let ExpressionNode::Simple(val_exp) = exp
                {
                    let mut values = Vec::new_in(&ctx.allocator);
                    let val_node = SimpleExpressionNode::from_node(val_exp);
                    values.push(Box::new_in(val_node, &ctx.allocator));

                    let set_dynamic = crate::ir::SetDynamicPropsIRNode {
                        element: element_id,
                        props: values,
                        is_event: true,
                    };
                    ctx.push_dynamic_operation(block, OperationNode::SetDynamicProps(set_dynamic));
                }
            }
        }
        "if" => {
            // v-if
            if let Some(ref exp) = dir.exp
                && let ExpressionNode::Simple(cond_exp) = exp
            {
                let cond_node = SimpleExpressionNode::from_node(cond_exp);
                let condition = Box::new_in(cond_node, &ctx.allocator);
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
                    .push(OperationNode::If(Box::new_in(if_node, &ctx.allocator)));
            }
        }
        "for" => {
            // v-for
            if let Some(ref exp) = dir.exp
                && let ExpressionNode::Simple(source_exp) = exp
            {
                let source_node = SimpleExpressionNode::from_node(source_exp);
                let source = Box::new_in(source_node, &ctx.allocator);
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
                    .push(OperationNode::For(Box::new_in(for_node, &ctx.allocator)));
            }
        }
        "html" => {
            // v-html
            if let Some(ref exp) = dir.exp
                && let ExpressionNode::Simple(val_exp) = exp
            {
                let val_node = SimpleExpressionNode::from_node(val_exp);
                let value = Box::new_in(val_node, &ctx.allocator);
                let set_html = SetHtmlIRNode {
                    element: element_id,
                    value,
                };

                ctx.push_dynamic_operation(block, OperationNode::SetHtml(set_html));
            }
        }
        "text" => {
            // v-text
            if let Some(ref exp) = dir.exp
                && let ExpressionNode::Simple(val_exp) = exp
            {
                let mut values = Vec::new_in(&ctx.allocator);
                let val_node = SimpleExpressionNode::from_node(val_exp);
                values.push(Box::new_in(val_node, &ctx.allocator));

                let set_text = SetTextIRNode {
                    element: element_id,
                    values,
                };

                ctx.push_dynamic_operation(block, OperationNode::SetText(set_text));
            }
        }
        "once" => {}
        "memo" => {}
        "show" => {
            // v-show - builtin directive
            let new_dir = clone_directive(ctx, dir);

            let dir_node = DirectiveIRNode {
                element: element_id,
                dir: Box::new_in(new_dir, &ctx.allocator),
                name: "vShow",
                builtin: true,
                tag: el.tag,
                input_type: get_static_attr(el, "type"),
            };

            block.operation.push(OperationNode::Directive(dir_node));
        }
        "cloak" => {
            let new_dir = clone_directive(ctx, dir);

            let dir_node = DirectiveIRNode {
                element: element_id,
                dir: Box::new_in(new_dir, &ctx.allocator),
                name: "vCloak",
                builtin: true,
                tag: el.tag,
                input_type: get_static_attr(el, "type"),
            };

            block.operation.push(OperationNode::Directive(dir_node));
        }
        "model" => {
            // v-model - builtin directive
            let new_dir = clone_directive(ctx, dir);

            let dir_node = DirectiveIRNode {
                element: element_id,
                dir: Box::new_in(new_dir, &ctx.allocator),
                name: "model",
                builtin: true,
                tag: el.tag,
                input_type: get_static_attr(el, "type"),
            };

            block.operation.push(OperationNode::Directive(dir_node));
        }
        _ => {
            // Custom directive - preserve the original payload for codegen parity.
            let new_dir = clone_directive(ctx, dir);

            let dir_node = DirectiveIRNode {
                element: element_id,
                dir: Box::new_in(new_dir, &ctx.allocator),
                name: dir.name,
                builtin: false,
                tag: el.tag,
                input_type: "",
            };

            block.operation.push(OperationNode::Directive(dir_node));
        }
    }
}

fn clone_directive<'a>(ctx: &TransformContext<'a>, dir: &DirectiveNode<'a>) -> DirectiveNode<'a> {
    let mut new_dir = DirectiveNode::new(ctx.allocator, dir.name, dir.loc.clone());
    new_dir.raw_name = dir.raw_name;
    new_dir.shorthand = dir.shorthand;
    new_dir.exp = clone_expression(ctx, dir.exp.as_ref());
    new_dir.arg = clone_expression(ctx, dir.arg.as_ref());

    for modifier in dir.modifiers.iter() {
        new_dir
            .modifiers
            .push(SimpleExpressionNode::from_node(modifier));
    }

    new_dir
}

fn clone_expression<'a>(
    ctx: &TransformContext<'a>,
    expr: Option<&ExpressionNode<'a>>,
) -> Option<ExpressionNode<'a>> {
    let expr = expr?;

    Some(match expr {
        ExpressionNode::Simple(simple) => {
            let cloned = SimpleExpressionNode::from_node(simple);
            ExpressionNode::Simple(Box::new_in(cloned, &ctx.allocator))
        }
        ExpressionNode::Compound(compound) => {
            let text = compound.loc.span.slice(ctx.source);
            let cloned = SimpleExpressionNode::new(text, false, compound.loc.clone());
            ExpressionNode::Simple(Box::new_in(cloned, &ctx.allocator))
        }
    })
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
        if let PropNode::Attribute(attr) = p
            && attr.name == "class"
            && let Some(ref value) = attr.value
        {
            return Some(value.content);
        }
        None
    });

    if let Some(static_val) = static_class {
        // Create a merged values list: the static class as the first entry
        let mut merged = Vec::new_in(&ctx.allocator);
        let static_node = SimpleExpressionNode::new(static_val, true, SourceLocation::STUB);
        merged.push(Box::new_in(static_node, &ctx.allocator));
        for v in dynamic_values.into_iter() {
            merged.push(v);
        }
        merged
    } else {
        dynamic_values
    }
}

/// Get a static attribute value from an element
fn get_static_attr<'a>(el: &ElementNode<'a>, attr_name: &str) -> &'a str {
    for prop in el.props.iter() {
        if let PropNode::Attribute(attr) = prop
            && attr.name == attr_name
            && let Some(ref value) = attr.value
        {
            return value.content;
        }
    }
    ""
}
