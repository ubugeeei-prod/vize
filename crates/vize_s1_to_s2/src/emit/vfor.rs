//! Native HTML `ui.for` (`v-for`) emission.

use vize_davinci::id::NodeId;
use vize_s0::{String, ToCompactString};
use vize_s2::expr::ExprRef;
use vize_s2::op::{Attribute, BindingOp, ForOp, Op, VueMemoOp};

use super::EmitCx;
use super::EmitError;
use super::UnsupportedReason as Reason;
use super::buf::Buf;
use super::helper::Helper;
use super::js::{RawJs, escape_js_string, expr_source, js_expr_source};

pub(super) fn emit_for(
    cx: &mut EmitCx<'_>,
    for_op: &ForOp<'_>,
    id: Option<NodeId>,
    fragment_key: Option<&str>,
) -> Result<(), EmitError> {
    let source_raw = js_source(&for_op.binding.source)?;
    let source = source_raw.as_str();
    let value = value_alias(&for_op.binding.value)?;
    let key_alias = optional_ident(&for_op.binding.key)?;
    let index_alias = optional_ident(&for_op.binding.index)?;
    let wrapper = id.and_then(|id| cx.for_wrappers.get(id));
    let wrapper_key = match wrapper.and_then(|wrapper| wrapper.key.as_ref()) {
        Some(key) => Some(super::tpl::wrapper_key_js(key)?),
        None => None,
    };
    let wrapper_attrs = wrapper
        .map(|wrapper| wrapper.attributes.as_slice())
        .unwrap_or(&[]);
    let wrapper_class = wrapper.and_then(|wrapper| wrapper.class.as_ref());
    let from_template = wrapper.is_some();
    let item_memo = if from_template {
        None
    } else {
        plain_item_memo(&for_op.region.ops)
    };
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
            _ => return Err(EmitError::unsupported_at(Reason::ForItemShape, for_op.span)),
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
    if item_memo.is_some() {
        if key_alias.is_none() {
            cx.buf.push(", __");
        }
        if index_alias.is_none() {
            cx.buf.push(", ___");
        }
        cx.buf.push(", _cached");
    }
    cx.buf.push(") => {");
    cx.buf.indent();
    cx.buf.newline();
    if let Some(memo) = item_memo {
        emit_memo_body(cx, for_op, id, bind_len, stable, memo)?;
        cx.buf.deindent();
        cx.buf.newline();
        cx.buf.push("}, _cache, ");
        let cache_index = super::memo::next_cache_index(cx);
        cx.buf.push(cache_index.as_str());
        cx.buf.push("), ");
        cx.buf.push(flag.to_compact_string().as_str());
        cx.buf.push(" /* ");
        cx.buf.push(flag_name);
        cx.buf.push(" */))");
        return Ok(());
    }
    cx.buf.push("return ");
    let prev_in_v_for = cx.in_v_for;
    let scope_mark = cx.push_scope(id);
    cx.in_v_for = true;
    let item = if from_template {
        super::tpl::emit_for_template_item(
            cx,
            &for_op.region.ops,
            stable,
            wrapper_key.as_deref(),
            wrapper_attrs,
            wrapper_class,
        )
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

fn emit_memo_body(
    cx: &mut EmitCx<'_>,
    for_op: &ForOp<'_>,
    id: Option<NodeId>,
    bind_len: usize,
    stable: bool,
    memo: &VueMemoOp<'_>,
) -> Result<(), EmitError> {
    let deps = super::memo::js_value(memo)?;
    let key = memo_item_key_js(&for_op.region.ops)?;
    cx.buf.use_helper(Helper::WithMemo);
    cx.buf.push("const _memo = (");
    cx.buf.push(deps.as_str());
    cx.buf.push(")");
    cx.buf.newline();
    cx.buf.use_helper(Helper::IsMemoSame);
    cx.buf.push("if (_cached && _cached.el && ");
    if let Some(key) = key {
        cx.buf.push("_cached.key === ");
        cx.buf.push(key.as_str());
        cx.buf.push(" && ");
    }
    cx.buf.push(Helper::IsMemoSame.alias());
    cx.buf.push("(_cached, _memo)) return _cached");
    cx.buf.newline();
    cx.buf.push("const _item = ");
    let prev_in_v_for = cx.in_v_for;
    let prev_skip_memo = cx.skip_memo;
    let scope_mark = cx.push_scope(id);
    cx.in_v_for = true;
    cx.skip_memo = true;
    let item = emit_plain_item(cx, for_op, bind_len, stable);
    cx.skip_memo = prev_skip_memo;
    cx.in_v_for = prev_in_v_for;
    cx.pop_scope(scope_mark);
    item?;
    cx.buf.newline();
    cx.buf.push("_item.memo = _memo");
    cx.buf.newline();
    cx.buf.push("return _item");
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
            super::emit_for_item_call(cx, element, id, stable, key.as_deref())
        }
        [Op::Component(component)] => {
            let key = item_key_js(&component.attributes, &component.bindings)?;
            super::component::emit_for_item(cx, component, id, key.as_deref())
        }
        [Op::Slot(slot)] => super::outlet::emit_outlet(cx, slot, None, false),
        _ => Err(EmitError::unsupported_at(Reason::ForItemShape, for_op.span)),
    }
}

fn plain_item_memo<'a>(ops: &'a [Op<'a>]) -> Option<&'a VueMemoOp<'a>> {
    match ops {
        [Op::Element(element)] => super::memo::first(&element.bindings),
        [Op::Component(component)] => super::memo::first(&component.bindings),
        _ => None,
    }
}

fn memo_item_key_js(ops: &[Op<'_>]) -> Result<Option<String>, EmitError> {
    match ops {
        [Op::Element(element)] => item_key_js(&element.attributes, &element.bindings),
        [Op::Component(component)] => item_key_js(&component.attributes, &component.bindings),
        _ => Ok(None),
    }
}

pub(super) fn js_source<'a>(expr: &'a ExprRef<'a>) -> Result<RawJs<'a>, EmitError> {
    expr_source(expr, false)
        .ok_or_else(|| EmitError::unsupported_at(Reason::ForSourceNotJs, expr.span()))
}

pub(super) fn value_alias<'a>(expr: &'a ExprRef<'a>) -> Result<&'a str, EmitError> {
    match expr {
        ExprRef::Js(js) if js.source.is_empty() => Ok("_item"),
        ExprRef::Js(js) => Ok(js.source),
        // `{ id = 1 }` is a pattern, not a JS expression; law 5 says
        // emit the authored source verbatim into the callback param.
        ExprRef::Opaque(opaque) if opaque.source.is_empty() => Ok("_item"),
        ExprRef::Opaque(opaque) => Ok(opaque.source),
        _ => Err(EmitError::unsupported_at(
            Reason::ForAliasNotEmittable,
            expr.span(),
        )),
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
            let source = js_expr_source(super::props::js_value(bind)?);
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

fn has_item_key(attributes: &[Attribute<'_>], bindings: &[BindingOp<'_>]) -> bool {
    attributes.iter().any(|attr| attr.name == "key")
        || bindings.iter().any(|binding| {
            matches!(
                binding,
                BindingOp::Bind(bind) if super::props_bind::is_key_bind_name(bind)
            )
        })
}
