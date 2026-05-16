//! Effect generation and inline operation helpers.

use crate::ir::{IREffect, OperationNode};
use vize_carton::{cstr, FxHashMap, String};

use super::{context::GenerateContext, operations::generate_operation, setup::is_svg_tag};

/// Generate effect
pub(crate) fn generate_effect(
    ctx: &mut GenerateContext,
    effect: &IREffect<'_>,
    element_template_map: &FxHashMap<usize, usize>,
) {
    ctx.use_helper("renderEffect");

    // If only one operation, use single-line format
    if effect.operations.len() == 1 {
        let op = &effect.operations[0];
        if let Some(op_code) = generate_operation_inline(ctx, op) {
            ctx.push_line_fmt(format_args!("_renderEffect(() => {op_code})"));
            return;
        }
    }

    ctx.push_line("_renderEffect(() => {");
    ctx.indent();

    for op in effect.operations.iter() {
        generate_operation(ctx, op, element_template_map);
    }

    ctx.deindent();
    ctx.push_line("})");
}

/// Generate operation inline (returns code string)
pub(crate) fn generate_operation_inline(
    ctx: &mut GenerateContext,
    op: &OperationNode<'_>,
) -> Option<String> {
    match op {
        OperationNode::SetProp(set_prop) => Some(generate_set_prop_inline(ctx, set_prop)),
        OperationNode::SetDynamicProps(set_props) => {
            Some(generate_set_dynamic_props_inline(ctx, set_props))
        }
        OperationNode::SetText(set_text) => {
            ctx.use_helper("setText");
            let text_ref = if let Some(text_var) = ctx.text_nodes.get(&set_text.element) {
                text_var.clone()
            } else {
                cstr!("n{}", set_text.element)
            };

            let values: Vec<String> = set_text
                .values
                .iter()
                .map(|v| {
                    if v.is_static {
                        cstr!("\"{}\"", escape_text_literal(v.content.as_str()))
                    } else {
                        ctx.use_helper("toDisplayString");
                        let resolved = ctx.resolve_expression(&v.content);
                        cstr!("_toDisplayString({})", resolved)
                    }
                })
                .collect();

            if values.len() == 1 {
                Some(cstr!("_setText({text_ref}, {})", values[0]))
            } else {
                Some(cstr!("_setText({text_ref}, {})", values.join(" + ")))
            }
        }
        _ => None,
    }
}

/// Generate SetProp inline (returns code string)
fn generate_set_prop_inline(
    ctx: &mut GenerateContext,
    set_prop: &crate::ir::SetPropIRNode<'_>,
) -> String {
    let element = cstr!("n{}", set_prop.element);
    let key = &set_prop.prop.key.content;
    let is_svg = is_svg_tag(set_prop.tag.as_str());

    // Build value from values list
    let value = build_prop_value(ctx, set_prop);

    if key.as_str() == "class" {
        if is_svg {
            ctx.use_helper("setAttr");
            cstr!("_setAttr({element}, \"class\", {value})")
        } else {
            ctx.use_helper("setClass");
            cstr!("_setClass({element}, {value})")
        }
    } else if key.as_str() == "style" {
        if is_svg {
            ctx.use_helper("setAttr");
            cstr!("_setAttr({element}, \"style\", {value})")
        } else {
            ctx.use_helper("setStyle");
            cstr!("_setStyle({element}, {value})")
        }
    } else if set_prop.prop_modifier {
        // .prop modifier -> setDOMProp
        ctx.use_helper("setDOMProp");
        cstr!("_setDOMProp({element}, \"{key}\", {value})")
    } else if set_prop.camel && is_svg {
        // .camel on SVG -> setAttr with true flag for SVG
        ctx.use_helper("setAttr");
        cstr!("_setAttr({element}, \"{key}\", {value}, true)")
    } else {
        ctx.use_helper("setProp");
        cstr!("_setProp({element}, \"{key}\", {value})")
    }
}

/// Build prop value expression, handling multiple values (static+dynamic merge)
fn build_prop_value(
    ctx: &mut GenerateContext,
    set_prop: &crate::ir::SetPropIRNode<'_>,
) -> vize_carton::CompactString {
    if set_prop.prop.values.len() > 1 {
        // Multiple values -> array notation (e.g. ["static", dynamic])
        let parts: Vec<String> = set_prop
            .prop
            .values
            .iter()
            .map(|v| {
                if v.is_static {
                    cstr!("\"{}\"", v.content)
                } else {
                    ctx.resolve_expression(&v.content)
                }
            })
            .collect();
        cstr!("[{}]", parts.join(", "))
    } else if let Some(first) = set_prop.prop.values.first() {
        if first.is_static {
            cstr!("\"{}\"", first.content)
        } else {
            ctx.resolve_expression(&first.content)
        }
    } else {
        vize_carton::CompactString::from("undefined")
    }
}

/// Generate SetDynamicProps inline (returns code string)
fn generate_set_dynamic_props_inline(
    ctx: &mut GenerateContext,
    set_props: &crate::ir::SetDynamicPropsIRNode<'_>,
) -> String {
    let element = cstr!("n{}", set_props.element);

    if set_props.is_event {
        ctx.use_helper("setDynamicEvents");
        if let Some(first) = set_props.props.first() {
            let resolved = ctx.resolve_expression(&first.content);
            cstr!("_setDynamicEvents({element}, {resolved})")
        } else {
            cstr!("_setDynamicEvents({element})")
        }
    } else {
        ctx.use_helper("setDynamicProps");
        let props_parts: Vec<String> = set_props
            .props
            .iter()
            .map(|p| {
                if p.is_static {
                    cstr!("\"{}\"", p.content)
                } else {
                    ctx.resolve_expression(&p.content)
                }
            })
            .collect();
        cstr!("_setDynamicProps({element}, [{}])", props_parts.join(", "))
    }
}

fn escape_text_literal(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
        .into()
}
