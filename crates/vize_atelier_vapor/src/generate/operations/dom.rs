use crate::ir::{SetDynamicPropsIRNode, SetHtmlIRNode, SetPropIRNode, SetTextIRNode};
use vize_carton::{cstr, String};

use super::super::{context::GenerateContext, setup::is_svg_tag};

/// Generate SetProp
pub(super) fn generate_set_prop(ctx: &mut GenerateContext, set_prop: &SetPropIRNode<'_>) {
    let element = cstr!("n{}", set_prop.element);
    let key = &set_prop.prop.key.content;
    let is_svg = is_svg_tag(set_prop.tag.as_str());

    // Build value handling multiple values (static+dynamic merge)
    let value = if set_prop.prop.values.len() > 1 {
        let parts: Vec<vize_carton::String> = set_prop
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
    };

    if key.as_str() == "class" {
        if is_svg {
            ctx.use_helper("setAttr");
            ctx.push_line_fmt(format_args!("_setAttr({element}, \"class\", {value})"));
        } else {
            ctx.use_helper("setClass");
            ctx.push_line_fmt(format_args!("_setClass({element}, {value})"));
        }
    } else if key.as_str() == "style" {
        if is_svg {
            ctx.use_helper("setAttr");
            ctx.push_line_fmt(format_args!("_setAttr({element}, \"style\", {value})"));
        } else {
            ctx.use_helper("setStyle");
            ctx.push_line_fmt(format_args!("_setStyle({element}, {value})"));
        }
    } else if set_prop.prop_modifier {
        ctx.use_helper("setDOMProp");
        ctx.push_line_fmt(format_args!("_setDOMProp({element}, \"{key}\", {value})"));
    } else if set_prop.camel && is_svg {
        ctx.use_helper("setAttr");
        ctx.push_line_fmt(format_args!(
            "_setAttr({element}, \"{key}\", {value}, true)"
        ));
    } else {
        ctx.use_helper("setProp");
        ctx.push_line_fmt(format_args!("_setProp({element}, \"{key}\", {value})"));
    }
}

/// Generate SetDynamicProps
pub(super) fn generate_set_dynamic_props(
    ctx: &mut GenerateContext,
    set_props: &SetDynamicPropsIRNode<'_>,
) {
    let element = cstr!("n{}", set_props.element);

    if set_props.is_event {
        // v-on="handlers" → _setDynamicEvents
        ctx.use_helper("setDynamicEvents");
        for prop in set_props.props.iter() {
            let resolved = ctx.resolve_expression(&prop.content);
            ctx.push_line_fmt(format_args!("_setDynamicEvents({}, {})", element, resolved));
        }
    } else {
        ctx.use_helper("setDynamicProps");
        let props_parts: std::vec::Vec<vize_carton::String> = set_props
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
        ctx.push_line_fmt(format_args!(
            "_setDynamicProps({}, [{}])",
            element,
            props_parts.join(", ")
        ));
    }
}

/// Generate SetText
pub(super) fn generate_set_text(ctx: &mut GenerateContext, set_text: &SetTextIRNode<'_>) {
    ctx.use_helper("setText");

    // Use text node reference if available, otherwise use element directly
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
                cstr!("\"{}\"", v.content)
            } else {
                ctx.use_helper("toDisplayString");
                let resolved = ctx.resolve_expression(&v.content);
                cstr!("_toDisplayString({})", resolved)
            }
        })
        .collect();

    if values.len() == 1 {
        ctx.push_line_fmt(format_args!("_setText({}, {})", text_ref, values[0]));
    } else {
        ctx.push_line_fmt(format_args!(
            "_setText({}, {})",
            text_ref,
            values.join(" + ")
        ));
    }
}

/// Generate SetHtml
pub(super) fn generate_set_html(ctx: &mut GenerateContext, set_html: &SetHtmlIRNode<'_>) {
    let element = cstr!("n{}", set_html.element);

    let value = if set_html.value.is_static {
        cstr!("\"{}\"", set_html.value.content)
    } else {
        ctx.resolve_expression(set_html.value.content.as_str())
    };

    ctx.push_line_fmt(format_args!("{}.innerHTML = {}", element, value));
}
