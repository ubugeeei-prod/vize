use crate::ir::{SetDynamicPropsIRNode, SetHtmlIRNode, SetPropIRNode, SetTextIRNode};
use vize_atelier_core::{SimpleExpressionNode, codegen::document::EmitDocument};
use vize_carton::cstr;

use super::super::{
    context::GenerateContext,
    setup::{escape_js_string_literal, is_svg_tag},
};

/// Generate SetProp
pub(super) fn generate_set_prop(ctx: &mut GenerateContext, set_prop: &SetPropIRNode<'_>) {
    let line = set_prop_call(ctx, set_prop);
    ctx.push_line_spanned(&line);
}

/// The `_setProp(...)`-family call for a SetProp, with its anchors; shared by
/// statement and inline-effect emission.
pub(crate) fn set_prop_call(
    ctx: &mut GenerateContext,
    set_prop: &SetPropIRNode<'_>,
) -> EmitDocument {
    let element = cstr!("n{}", set_prop.element);
    let key = &set_prop.prop.key.content;
    let is_svg = is_svg_tag(set_prop.tag);

    // Build value handling multiple values (static+dynamic merge)
    let value = if set_prop.prop.values.len() > 1 {
        let mut list = EmitDocument::plain("[");
        for (index, v) in set_prop.prop.values.iter().enumerate() {
            if index > 0 {
                list.push_str(", ");
            }
            list.push_spanned(&prop_value(ctx, v));
        }
        list.push_str("]");
        list
    } else if let Some(first) = set_prop.prop.values.first() {
        prop_value(ctx, first)
    } else {
        EmitDocument::plain("undefined")
    };
    // The key maps to the authored `v-bind` argument it copies.
    let key_loc = set_prop.prop.key.loc.span;
    let named = ctx.spanned_at(key, (key_loc.start < key_loc.end).then_some(key_loc.start));
    let call = |callee: &str, key: Option<&EmitDocument>, suffix: &str| {
        let mut line = EmitDocument::plain(callee);
        line.push_str("(");
        line.push_str(&element);
        if let Some(key) = key {
            line.push_str(", \"");
            line.push_spanned(key);
            line.push_str("\"");
        }
        line.push_str(", ");
        line.push_spanned(&value);
        line.push_str(suffix);
        line.push_str(")");
        line
    };

    let (helper, line) = if (*key == "class" || *key == "style") && is_svg {
        ("setAttr", call("_setAttr", Some(&named), ""))
    } else if *key == "class" {
        ("setClass", call("_setClass", None, ""))
    } else if *key == "style" {
        ("setStyle", call("_setStyle", None, ""))
    } else if set_prop.prop_modifier {
        ("setDOMProp", call("_setDOMProp", Some(&named), ""))
    } else if set_prop.camel && is_svg {
        ("setAttr", call("_setAttr", Some(&named), ", true"))
    } else {
        ("setProp", call("_setProp", Some(&named), ""))
    };
    ctx.use_helper(helper);
    line
}

/// One bound value: a static literal, or an expression with its anchors.
fn prop_value(ctx: &GenerateContext, value: &SimpleExpressionNode<'_>) -> EmitDocument {
    if value.is_static {
        EmitDocument::plain(&cstr!("\"{}\"", escape_js_string_literal(value.content)))
    } else {
        ctx.spanned_expression_node(value)
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
            let resolved = ctx.resolve_expression_node(prop);
            ctx.push_line_fmt(format_args!("_setDynamicEvents({}, {})", element, resolved));
        }
    } else {
        ctx.use_helper("setDynamicProps");
        let props_parts: std::vec::Vec<vize_carton::String> = set_props
            .props
            .iter()
            .map(|p| {
                if p.is_static {
                    cstr!("\"{}\"", escape_js_string_literal(p.content))
                } else {
                    ctx.resolve_expression_node(p)
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
    let line = set_text_call(ctx, set_text);
    ctx.push_line_spanned(&line);
}

/// The `_setText(...)` call for a SetText, with its anchors; shared by
/// statement and inline-effect emission.
pub(crate) fn set_text_call(
    ctx: &mut GenerateContext,
    set_text: &SetTextIRNode<'_>,
) -> EmitDocument {
    let helper = if set_text.is_element {
        "setElementText"
    } else {
        "setText"
    };
    ctx.use_helper(helper);

    // Use text node reference if available, otherwise use element directly
    let text_ref = if !set_text.is_element
        && let Some(text_var) = ctx.text_nodes.get(&set_text.element)
    {
        text_var.clone()
    } else {
        cstr!("n{}", set_text.element)
    };

    let mut line = EmitDocument::plain("_");
    line.push_str(helper);
    line.push_str("(");
    line.push_str(&text_ref);
    line.push_str(", ");
    for (index, v) in set_text.values.iter().enumerate() {
        if index > 0 {
            line.push_str(" + ");
        }
        if v.is_static {
            let span = v.loc.span;
            line.push_str("\"");
            let text = escape_js_string_literal(v.content);
            line.push_spanned(
                &ctx.spanned_at(&text, (span.start < span.end).then_some(span.start)),
            );
            line.push_str("\"");
        } else {
            ctx.use_helper("toDisplayString");
            line.push_str("_toDisplayString(");
            line.push_spanned(&ctx.spanned_expression_node(v));
            line.push_str(")");
        }
    }
    line.push_str(")");
    line
}

/// Generate SetHtml
pub(super) fn generate_set_html(ctx: &mut GenerateContext, set_html: &SetHtmlIRNode<'_>) {
    let element = cstr!("n{}", set_html.element);

    let value = if set_html.value.is_static {
        cstr!("\"{}\"", escape_js_string_literal(set_html.value.content))
    } else {
        ctx.resolve_expression_node(&set_html.value)
    };

    ctx.push_line_fmt(format_args!("{}.innerHTML = {}", element, value));
}
