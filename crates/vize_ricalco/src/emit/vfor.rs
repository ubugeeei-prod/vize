//! Native HTML `ui.for` (`v-for`) emission.

use vize_carton::{String, ToCompactString};
use vize_davinci::id::NodeId;
use vize_disegno::expr::ExprRef;
use vize_disegno::op::{Attribute, BindingOp, ForOp, Op};

use super::EmitCx;
use super::EmitError;
use super::buf::Buf;
use super::js::escape_js_string;

pub(super) fn emit_for(
    cx: &mut EmitCx<'_>,
    for_op: &ForOp<'_>,
    id: Option<NodeId>,
    fragment_key: Option<&str>,
) -> Result<(), EmitError> {
    let source = js_source(&for_op.binding.source)?;
    let value = value_alias(&for_op.binding.value)?;
    let key_alias = optional_ident(&for_op.binding.key)?;
    let index_alias = optional_ident(&for_op.binding.index)?;
    let wrapper = id.and_then(|id| cx.for_wrappers.get(id));
    let wrapper_key = match wrapper.and_then(|wrapper| wrapper.key.as_ref()) {
        Some(key) => Some(super::tpl::wrapper_key_js(key)?),
        None => None,
    };
    let from_template = wrapper.is_some();
    let (bind_len, keyed) = if from_template {
        (0, wrapper_key.is_some())
    } else {
        match for_op.region.ops.as_slice() {
            [Op::Element(element)] => (
                element.bindings.len(),
                has_item_key(&element.attributes, &element.bindings),
            ),
            [Op::Component(component)] => (
                component.bindings.len(),
                has_item_key(&component.attributes, &component.bindings),
            ),
            [Op::Slot(slot)] => (
                slot.bindings.len(),
                has_item_key(&slot.attributes, &slot.bindings),
            ),
            _ => return Err(EmitError::Unsupported),
        }
    };
    let stable = is_numeric(source);
    let flag = if stable {
        64
    } else if keyed {
        128
    } else {
        256
    };
    let flag_name = match flag {
        64 => "STABLE_FRAGMENT",
        128 => "KEYED_FRAGMENT",
        256 => "UNKEYED_FRAGMENT",
        _ => "FRAGMENT",
    };

    cx.buf.use_open_block();
    cx.buf.use_create_element_block();
    cx.buf.use_fragment();
    cx.buf.use_render_list();
    cx.buf.push("(");
    cx.buf.push(Buf::open_block_alias());
    if stable {
        cx.buf.push("(), ");
    } else {
        cx.buf.push("(true), ");
    }
    cx.buf.push(Buf::create_element_block_alias());
    cx.buf.push("(");
    cx.buf.push(Buf::fragment_alias());
    if let Some(key) = fragment_key {
        cx.buf.push(", { key: ");
        cx.buf.push(key);
        cx.buf.push(" }, ");
    } else {
        cx.buf.push(", null, ");
    }
    cx.buf.push(Buf::render_list_alias());
    cx.buf.push("(");
    cx.buf.push(source);
    cx.buf.push(", (");
    cx.buf.push(value);
    if let Some(alias) = key_alias {
        cx.buf.push(", ");
        cx.buf.push(alias);
    }
    if let Some(alias) = index_alias {
        cx.buf.push(", ");
        cx.buf.push(alias);
    }
    cx.buf.push(") => {");
    cx.buf.indent();
    cx.buf.newline();
    cx.buf.push("return ");
    let prev_in_v_for = cx.in_v_for;
    let scope_mark = cx.push_scope(id);
    cx.in_v_for = true;
    let item = if from_template {
        super::tpl::emit_for_template_item(cx, &for_op.region.ops, stable, wrapper_key.as_deref())
    } else {
        emit_plain_item(cx, for_op, bind_len, stable)
    };
    cx.in_v_for = prev_in_v_for;
    cx.pop_scope(scope_mark);
    item?;
    cx.buf.deindent();
    cx.buf.newline();
    cx.buf.push("}), ");
    cx.buf.push(flag.to_compact_string().as_str());
    cx.buf.push(" /* ");
    cx.buf.push(flag_name);
    cx.buf.push(" */))");
    Ok(())
}

fn emit_plain_item(
    cx: &mut EmitCx<'_>,
    for_op: &ForOp<'_>,
    bind_len: usize,
    stable: bool,
) -> Result<(), EmitError> {
    let id = cx.walk.mint();
    cx.walk.skip(bind_len);
    match for_op.region.ops.as_slice() {
        [Op::Element(element)] => {
            let key = item_key_js(&element.attributes, &element.bindings)?;
            super::emit_for_item_call(cx, element, stable, key.as_deref())
        }
        [Op::Component(component)] => {
            let key = item_key_js(&component.attributes, &component.bindings)?;
            super::component::emit_for_item(cx, component, id, key.as_deref())
        }
        [Op::Slot(slot)] => super::outlet::emit_outlet(cx, slot, None, false),
        _ => Err(EmitError::Unsupported),
    }
}

pub(super) fn js_source<'a>(expr: &'a ExprRef<'a>) -> Result<&'a str, EmitError> {
    match expr {
        ExprRef::Js(js) => Ok(js.source),
        _ => Err(EmitError::Unsupported),
    }
}

pub(super) fn value_alias<'a>(expr: &'a ExprRef<'a>) -> Result<&'a str, EmitError> {
    match expr {
        ExprRef::Js(js) if js.source.is_empty() => Ok("_item"),
        ExprRef::Js(js) => Ok(js.source),
        // `{ id = 1 }` is a pattern, not a JS expression; law 5 says
        // emit the authored source verbatim into the callback param.
        ExprRef::Opaque(opaque) if opaque.source.is_empty() => Ok("_item"),
        ExprRef::Opaque(opaque) => Ok(opaque.source),
        _ => Err(EmitError::Unsupported),
    }
}

pub(super) fn optional_ident<'a>(
    expr: &'a Option<ExprRef<'a>>,
) -> Result<Option<&'a str>, EmitError> {
    match expr {
        None => Ok(None),
        Some(expr) => Ok(Some(value_alias(expr)?)),
    }
}

fn is_numeric(source: &str) -> bool {
    !source.is_empty() && source.chars().all(|c| c.is_ascii_digit())
}

fn item_key_js(
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
) -> Result<Option<String>, EmitError> {
    for binding in bindings {
        if let BindingOp::Bind(bind) = binding
            && super::props_bind::is_key_bind_name(bind)
        {
            return Ok(Some(String::from(super::props::js_value(bind)?.source)));
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

fn has_item_key(attributes: &[Attribute<'_>], bindings: &[BindingOp<'_>]) -> bool {
    attributes.iter().any(|attr| attr.name == "key")
        || bindings.iter().any(|binding| {
            matches!(
                binding,
                BindingOp::Bind(bind) if super::props_bind::is_key_bind_name(bind)
            )
        })
}
