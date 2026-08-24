//! Native HTML `ui.for` (`v-for`) emission.

use vize_carton::ToCompactString;
use vize_disegno::expr::ExprRef;
use vize_disegno::op::{Attribute, BindingOp, DynamicName, ForOp, Op};

use super::buf::Buf;
use super::js::is_valid_js_identifier;
use super::EmitCx;
use super::EmitError;

pub(super) fn emit_for(cx: &mut EmitCx<'_>, for_op: &ForOp<'_>) -> Result<(), EmitError> {
    let source = js_source(&for_op.binding.source)?;
    let value = value_alias(&for_op.binding.value)?;
    let key_alias = optional_ident(&for_op.binding.key)?;
    let index_alias = optional_ident(&for_op.binding.index)?;
    let (bind_len, keyed) = match for_op.region.ops.as_slice() {
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
    cx.buf.push(", null, ");
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
    let _id = cx.walk.mint();
    cx.walk.skip(bind_len);
    let prev_in_v_for = cx.in_v_for;
    cx.in_v_for = true;
    let item = match for_op.region.ops.as_slice() {
        [Op::Element(element)] if super::hoist::is_hoistable(element) => {
            let alias = super::hoist::hoist_static_element(cx, element);
            cx.buf.push(alias.as_str());
            Ok(())
        }
        [Op::Element(element)] => super::emit_for_item_call(cx, element, stable),
        [Op::Component(component)] => super::component::emit_for_item(cx, component, _id),
        [Op::Slot(slot)] => super::outlet::emit_outlet(cx, slot, None, false),
        _ => Err(EmitError::Unsupported),
    };
    cx.in_v_for = prev_in_v_for;
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

pub(super) fn js_source<'a>(expr: &'a ExprRef<'a>) -> Result<&'a str, EmitError> {
    match expr {
        ExprRef::Js(js) => Ok(js.source),
        _ => Err(EmitError::Unsupported),
    }
}

pub(super) fn value_alias<'a>(expr: &'a ExprRef<'a>) -> Result<&'a str, EmitError> {
    match expr {
        ExprRef::Js(js) if js.source.is_empty() => Ok("_item"),
        ExprRef::Js(js) if is_valid_js_identifier(js.source) => Ok(js.source),
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

fn has_item_key(attributes: &[Attribute<'_>], bindings: &[BindingOp<'_>]) -> bool {
    attributes.iter().any(|attr| attr.name == "key")
        || bindings.iter().any(|binding| {
            matches!(
                binding,
                BindingOp::Bind(bind) if matches!(bind.name, Some(DynamicName::Static("key")))
            )
        })
}
