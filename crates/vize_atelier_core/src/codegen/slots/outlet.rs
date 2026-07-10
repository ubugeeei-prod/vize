//! Slot outlet (`<slot />`) name and props generation.

use crate::relief_projection::ReliefRenderOp;
use crate::{DirectiveNode, ElementNode, ExpressionNode, PropNode, RuntimeHelper};
use vize_carton::String;

use super::super::context::CodegenContext;
use super::super::expression::generate_expression;
use super::super::helpers::{camelize, escape_js_string, is_valid_js_identifier};
use super::super::props::{
    generate_slot_outlet_directive_prop_with_static, generate_vbind_object_exp,
    generate_von_object_exp, is_supported_directive,
};

pub(crate) enum SlotOutletName<'a> {
    Static(String),
    Dynamic(&'a ExpressionNode<'a>),
}

fn is_slot_name_bind(dir: &DirectiveNode<'_>) -> bool {
    matches!(
        ReliefRenderOp::from_directive(dir),
        ReliefRenderOp::Directive { name: "bind", arg: Some(arg), .. }
            if matches!(arg.node(), Some(ExpressionNode::Simple(exp)) if exp.is_static && exp.content == "name")
    )
}

fn is_slot_name_prop(prop: &PropNode<'_>) -> bool {
    match ReliefRenderOp::from_prop(prop) {
        ReliefRenderOp::Attribute { name: "name", .. } => true,
        ReliefRenderOp::Directive { .. } => {
            matches!(prop, PropNode::Directive(dir) if is_slot_name_bind(dir))
        }
        _ => false,
    }
}

fn is_slot_outlet_object_spread(prop: &PropNode<'_>) -> bool {
    matches!(
        ReliefRenderOp::from_prop(prop),
        ReliefRenderOp::Directive {
            name: "bind" | "on",
            arg: None,
            exp: Some(_),
            ..
        }
    )
}

fn slot_outlet_prop_generates_output(prop: &PropNode<'_>) -> bool {
    if is_slot_name_prop(prop) {
        return false;
    }

    match ReliefRenderOp::from_prop(prop) {
        ReliefRenderOp::Attribute { .. } => true,
        ReliefRenderOp::Directive { name, arg, exp, .. } => {
            if matches!(name, "bind" | "on") && arg.is_none() {
                return exp.is_some();
            }
            let PropNode::Directive(dir) = prop else {
                unreachable!("Relief projection directive must borrow a directive prop");
            };
            is_supported_directive(dir)
        }
        _ => false,
    }
}

fn has_slot_outlet_vbind_object(el: &ElementNode<'_>) -> bool {
    el.props.iter().any(|prop| {
        matches!(
            ReliefRenderOp::from_prop(prop),
            ReliefRenderOp::Directive {
                name: "bind",
                arg: None,
                exp: Some(_),
                ..
            }
        )
    })
}

fn has_slot_outlet_von_object(el: &ElementNode<'_>) -> bool {
    el.props.iter().any(|prop| {
        matches!(
            ReliefRenderOp::from_prop(prop),
            ReliefRenderOp::Directive {
                name: "on",
                arg: None,
                exp: Some(_),
                ..
            }
        )
    })
}

fn has_slot_outlet_entry_props(el: &ElementNode<'_>) -> bool {
    el.props
        .iter()
        .any(|prop| !is_slot_outlet_object_spread(prop) && slot_outlet_prop_generates_output(prop))
}

pub(crate) fn get_slot_outlet_name<'a>(el: &'a ElementNode<'a>) -> SlotOutletName<'a> {
    for prop in &el.props {
        match ReliefRenderOp::from_prop(prop) {
            ReliefRenderOp::Attribute {
                name: "name",
                value,
                ..
            } => {
                let name = value
                    .map(String::new)
                    .unwrap_or_else(|| String::new("default"));
                return SlotOutletName::Static(name);
            }
            ReliefRenderOp::Directive { exp, .. } if matches!(prop, PropNode::Directive(dir) if is_slot_name_bind(dir)) => {
                if let Some(exp) = exp.and_then(|exp| exp.node()) {
                    return SlotOutletName::Dynamic(exp);
                }
            }
            _ => {}
        }
    }

    SlotOutletName::Static(String::new("default"))
}

pub(crate) fn generate_slot_outlet_name(ctx: &mut CodegenContext, el: &ElementNode<'_>) {
    match get_slot_outlet_name(el) {
        SlotOutletName::Static(name) => {
            ctx.push("\"");
            ctx.push(&escape_js_string(name.as_str()));
            ctx.push("\"");
        }
        SlotOutletName::Dynamic(exp) => generate_expression(ctx, exp),
    }
}

pub(crate) fn has_slot_outlet_props(el: &ElementNode<'_>) -> bool {
    el.props.iter().any(slot_outlet_prop_generates_output)
}

pub(crate) fn generate_slot_outlet_props_entries(ctx: &mut CodegenContext, el: &ElementNode<'_>) {
    let static_merge = crate::codegen::props::StaticMerge::from_props(&el.props);

    let has_dynamic_class = el.props.iter().any(|prop| match ReliefRenderOp::from_prop(prop) {
        ReliefRenderOp::Directive { name: "bind", arg: Some(arg), .. } => {
            !is_slot_name_prop(prop)
                && matches!(arg.node(), Some(ExpressionNode::Simple(exp)) if exp.is_static && exp.content == "class")
        }
        _ => false,
    });

    let has_dynamic_style = el.props.iter().any(|prop| match ReliefRenderOp::from_prop(prop) {
        ReliefRenderOp::Directive { name: "bind", arg: Some(arg), .. } => {
            !is_slot_name_prop(prop)
                && matches!(arg.node(), Some(ExpressionNode::Simple(exp)) if exp.is_static && exp.content == "style")
        }
        _ => false,
    });

    let mut first = true;
    for prop in &el.props {
        if is_slot_name_prop(prop) {
            continue;
        }

        match ReliefRenderOp::from_prop(prop) {
            ReliefRenderOp::Attribute {
                name,
                name_span,
                value,
                value_span,
                ..
            } => {
                if (name == "class" && has_dynamic_class) || (name == "style" && has_dynamic_style)
                {
                    continue;
                }

                if !first {
                    ctx.push(", ");
                }

                let key = camelize(name);
                ctx.record_mapping_named(&name_span.start, name);
                if is_valid_js_identifier(&key) {
                    ctx.push(&key);
                } else {
                    ctx.push("\"");
                    ctx.push(&escape_js_string(&key));
                    ctx.push("\"");
                }
                ctx.push(": ");
                if let Some(value) = value {
                    ctx.push("\"");
                    if let Some(span) = value_span {
                        ctx.record_mapping(&span.start);
                    }
                    ctx.push(&escape_js_string(value));
                    ctx.push("\"");
                } else {
                    ctx.push("\"\"");
                }
                first = false;
            }
            ReliefRenderOp::Directive { name, arg, .. } => {
                let PropNode::Directive(dir) = prop else {
                    unreachable!("Relief projection directive must borrow a directive prop");
                };
                if !is_supported_directive(dir) || (arg.is_none() && matches!(name, "bind" | "on"))
                {
                    continue;
                }

                if !first {
                    ctx.push(", ");
                }
                generate_slot_outlet_directive_prop_with_static(ctx, dir, static_merge);
                first = false;
            }
            _ => unreachable!(
                "element props lower to attribute or directive Relief projection operations"
            ),
        }
    }
}

pub(crate) fn generate_slot_outlet_props(ctx: &mut CodegenContext, el: &ElementNode<'_>) {
    generate_slot_outlet_props_inner(ctx, el, None);
}

pub(crate) fn generate_slot_outlet_props_with_key(
    ctx: &mut CodegenContext,
    el: &ElementNode<'_>,
    generate_key: &dyn Fn(&mut CodegenContext),
) {
    generate_slot_outlet_props_inner(ctx, el, Some(generate_key));
}

fn generate_slot_outlet_props_inner(
    ctx: &mut CodegenContext,
    el: &ElementNode<'_>,
    generate_key: Option<&dyn Fn(&mut CodegenContext)>,
) {
    let has_vbind_object = has_slot_outlet_vbind_object(el);
    let has_von_object = has_slot_outlet_von_object(el);
    let has_entries = has_slot_outlet_entry_props(el);
    let has_key = generate_key.is_some();

    if has_vbind_object || has_von_object {
        let needs_merge = has_key || has_entries || (has_vbind_object && has_von_object);

        if needs_merge {
            ctx.use_helper(RuntimeHelper::MergeProps);
            ctx.push(ctx.helper(RuntimeHelper::MergeProps));
            ctx.push("(");
            let mut first = true;

            if has_vbind_object {
                generate_vbind_object_exp(ctx, &el.props);
                first = false;
            }

            if has_von_object {
                if !first {
                    ctx.push(", ");
                }
                generate_von_object_exp(ctx, &el.props);
                first = false;
            }

            if has_key || has_entries {
                if !first {
                    ctx.push(", ");
                }
                generate_slot_outlet_props_object(ctx, el, generate_key, has_entries);
            }

            ctx.push(")");
        } else if has_vbind_object {
            ctx.use_helper(RuntimeHelper::NormalizeProps);
            ctx.use_helper(RuntimeHelper::GuardReactiveProps);
            ctx.push(ctx.helper(RuntimeHelper::NormalizeProps));
            ctx.push("(");
            ctx.push(ctx.helper(RuntimeHelper::GuardReactiveProps));
            ctx.push("(");
            generate_vbind_object_exp(ctx, &el.props);
            ctx.push("))");
        } else {
            generate_von_object_exp(ctx, &el.props);
        }
        return;
    }

    generate_slot_outlet_props_object(ctx, el, generate_key, has_entries);
}

fn generate_slot_outlet_props_object(
    ctx: &mut CodegenContext,
    el: &ElementNode<'_>,
    generate_key: Option<&dyn Fn(&mut CodegenContext)>,
    has_entries: bool,
) {
    ctx.push("{");
    let mut needs_separator = false;

    if let Some(generate_key) = generate_key {
        ctx.push(" key: ");
        generate_key(ctx);
        needs_separator = true;
    }

    if has_entries {
        if needs_separator {
            ctx.push(", ");
        } else {
            ctx.push(" ");
        }
        generate_slot_outlet_props_entries(ctx, el);
        needs_separator = true;
    }

    if needs_separator {
        ctx.push(" ");
    }
    ctx.push("}");
}
