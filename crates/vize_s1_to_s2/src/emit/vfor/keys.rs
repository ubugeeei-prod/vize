//! `v-for` item `:key` sources and the keyed-fragment decision.

use vize_s0::String;
use vize_s2::op::{Attribute, BindingOp, Op};

use super::super::js::{escape_js_string, js_expr_source};
use super::super::prefix::Site;
use super::super::{EmitCx, EmitError};

pub(super) fn memo_item_key_js(
    cx: &EmitCx<'_>,
    ops: &[Op<'_>],
) -> Result<Option<String>, EmitError> {
    match ops {
        [Op::Element(element)] => item_key_js(cx, &element.attributes, &element.bindings),
        [Op::Component(component)] => item_key_js(cx, &component.attributes, &component.bindings),
        _ => Ok(None),
    }
}

pub(super) fn item_key_js(
    cx: &EmitCx<'_>,
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
) -> Result<Option<String>, EmitError> {
    for binding in bindings {
        if let BindingOp::Bind(bind) = binding
            && crate::emit::props_bind::is_key_bind_name(bind)
        {
            let js = crate::emit::props::js_value(bind)?;
            if cx.prefixing() {
                return Ok(Some(cx.prefixed_js(js, Site::Expression)?));
            }
            let source = js_expr_source(js);
            return Ok(Some(String::from(source.as_str())));
        }
    }
    for attr in attributes {
        if attr.name == "key" {
            let mut out = String::from("\"");
            out.push_str(escape_js_string(attr.value.unwrap_or("")).as_str());
            out.push('"');
            return Ok(Some(out));
        }
    }
    Ok(None)
}

pub(super) fn has_item_key(attributes: &[Attribute<'_>], bindings: &[BindingOp<'_>]) -> bool {
    attributes.iter().any(|attr| attr.name == "key")
        || bindings.iter().any(|binding| {
            matches!(
                binding,
                BindingOp::Bind(bind) if crate::emit::props_bind::is_key_bind_name(bind)
            )
        })
}
