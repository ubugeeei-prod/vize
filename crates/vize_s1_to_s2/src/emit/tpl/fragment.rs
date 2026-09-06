use vize_s2::expr::{ExprRef, OpaqueReason};
use vize_s2::op::{InterpolationOp, Op};

use super::super::buf::Buf;
use super::super::children::{
    emit_comment_vnode, emit_create_text_vnode, emit_dynamic_part, emit_interpolation,
    emit_js_to_display_string, emit_plain_text_vnode, emit_raw_interpolation_or_refuse,
    emit_to_display_string, is_empty_interpolation,
};
use super::super::hoist::{emit_hoisted_element, is_hoistable};
use super::super::js::{escape_js_string, is_valid_js_identifier};
use super::super::prefix::Site;
use super::super::vnode;
use super::super::{EmitCx, EmitError, UnsupportedReason as Reason};
use crate::lower::{WrapperAttr, WrapperClass};

#[derive(Clone, Copy)]
pub(super) enum ChildMode {
    /// `generate_children_force_array`: interpolations are
    /// `_createTextVNode(_toDisplayString(...), 1)`.
    ForceArray,
    /// `generate_node` per child: interpolations stay bare `_toDisplayString`.
    GenerateNode,
}

pub(in crate::emit) fn emit_inline(cx: &mut EmitCx<'_>, ops: &[Op<'_>]) -> Result<(), EmitError> {
    match ops {
        [op] => emit_inline_child(cx, op),
        ops => {
            cx.buf.push("[");
            for (i, op) in ops.iter().enumerate() {
                if i > 0 {
                    cx.buf.push(", ");
                }
                emit_inline_child(cx, op)?;
            }
            cx.buf.push("]");
            Ok(())
        }
    }
}

pub(super) fn emit_inner_fragment(
    cx: &mut EmitCx<'_>,
    ops: &[Op<'_>],
    key: Option<&str>,
    attributes: &[WrapperAttr],
    class: Option<&WrapperClass>,
    mode: ChildMode,
) -> Result<(), EmitError> {
    cx.buf.use_open_block();
    cx.buf.use_create_element_block();
    cx.buf.use_fragment();
    cx.buf.push("(");
    cx.buf.push(Buf::open_block_alias());
    cx.buf.push("(), ");
    cx.buf.push(Buf::create_element_block_alias());
    cx.buf.push("(");
    cx.buf.push(Buf::fragment_alias());
    emit_fragment_props(cx, key, attributes, class)?;
    cx.with_static_vnode_hoist(true, |cx| emit_fragment_children(cx, ops, mode))?;
    cx.buf.push(", 64 /* STABLE_FRAGMENT */))");
    Ok(())
}

fn emit_fragment_props(
    cx: &mut EmitCx<'_>,
    key: Option<&str>,
    attributes: &[WrapperAttr],
    class: Option<&WrapperClass>,
) -> Result<(), EmitError> {
    if key.is_none() && attributes.is_empty() && class.is_none() {
        cx.buf.push(", null, ");
        return Ok(());
    }
    let multiline = class.is_some_and(|class| class.dynamic_source.is_some());
    if multiline {
        cx.buf.push(", {");
        cx.buf.indent();
    } else {
        cx.buf.push(", { ");
    }
    let mut first = true;
    if let Some(key) = key {
        start_fragment_prop(cx, &mut first, multiline);
        cx.buf.push("key: ");
        cx.buf.push(key);
    }
    for attr in attributes {
        start_fragment_prop(cx, &mut first, multiline);
        push_static_key(cx, attr.name.as_str());
        cx.buf.push(": \"");
        cx.buf
            .push(escape_js_string(attr.value.as_deref().unwrap_or("")).as_str());
        cx.buf.push("\"");
    }
    if let Some(class) = class {
        start_fragment_prop(cx, &mut first, multiline);
        cx.buf.push("class: ");
        emit_wrapper_class(cx, class)?;
    }
    if multiline {
        cx.buf.deindent();
        cx.buf.newline();
        cx.buf.push("}, ");
    } else {
        cx.buf.push(" }, ");
    }
    Ok(())
}

fn start_fragment_prop(cx: &mut EmitCx<'_>, first: &mut bool, multiline: bool) {
    if !*first {
        cx.buf.push(",");
        if !multiline {
            cx.buf.push(" ");
        }
    }
    if multiline {
        cx.buf.newline();
    }
    *first = false;
}

fn emit_wrapper_class(cx: &mut EmitCx<'_>, class: &WrapperClass) -> Result<(), EmitError> {
    let prefixed;
    let dynamic_source = match &class.dynamic_source {
        Some(source) if cx.prefixing() => {
            prefixed = cx.prefixed_text(source.as_str(), super::super::prefix::Site::Expression)?;
            Some(prefixed.as_str())
        }
        Some(source) => Some(source.as_str()),
        None => None,
    };
    match (&class.static_value, dynamic_source) {
        (Some(static_value), Some(dynamic_source)) => {
            cx.buf
                .use_helper(super::super::helper::Helper::NormalizeClass);
            cx.buf
                .push(super::super::helper::Helper::NormalizeClass.alias());
            cx.buf.push("([\"");
            cx.buf
                .push(escape_js_string(static_value.as_str()).as_str());
            cx.buf.push("\", ");
            cx.buf.push(dynamic_source);
            cx.buf.push("])");
        }
        (None, Some(dynamic_source)) => {
            cx.buf
                .use_helper(super::super::helper::Helper::NormalizeClass);
            cx.buf
                .push(super::super::helper::Helper::NormalizeClass.alias());
            cx.buf.push("(");
            cx.buf.push(dynamic_source);
            cx.buf.push(")");
        }
        (Some(static_value), None) => {
            cx.buf.push("\"");
            cx.buf
                .push(escape_js_string(static_value.as_str()).as_str());
            cx.buf.push("\"");
        }
        (None, None) => cx.buf.push("\"\""),
    }
    Ok(())
}

fn push_static_key(cx: &mut EmitCx<'_>, key: &str) {
    if is_valid_js_identifier(key) {
        cx.buf.push(key);
        return;
    }
    cx.buf.push("\"");
    cx.buf.push(escape_js_string(key).as_str());
    cx.buf.push("\"");
}

fn emit_fragment_children(
    cx: &mut EmitCx<'_>,
    ops: &[Op<'_>],
    mode: ChildMode,
) -> Result<(), EmitError> {
    if ops.is_empty() {
        cx.buf.push("null");
        return Ok(());
    }
    cx.buf.push("[");
    cx.buf.indent();
    match mode {
        ChildMode::ForceArray => emit_force_array(cx, ops)?,
        ChildMode::GenerateNode => emit_generate_node(cx, ops)?,
    }
    cx.buf.deindent();
    cx.buf.newline();
    cx.buf.push("]");
    Ok(())
}

fn emit_force_array(cx: &mut EmitCx<'_>, ops: &[Op<'_>]) -> Result<(), EmitError> {
    let mut i = 0;
    let mut first = true;
    while i < ops.len() {
        if matches!(ops[i], Op::Text(_) | Op::Interpolation(_)) {
            let start = i;
            while i < ops.len() && matches!(ops[i], Op::Text(_) | Op::Interpolation(_)) {
                i += 1;
            }
            start_item(cx, &mut first);
            emit_create_text_vnode(cx, &ops[start..i])?;
            continue;
        }
        start_item(cx, &mut first);
        emit_node_child(cx, &ops[i])?;
        i += 1;
    }
    Ok(())
}

fn emit_generate_node(cx: &mut EmitCx<'_>, ops: &[Op<'_>]) -> Result<(), EmitError> {
    let mut first = true;
    for op in ops {
        match op {
            Op::Text(_) => {
                start_item(cx, &mut first);
                emit_create_text_vnode(cx, core::slice::from_ref(op))?;
            }
            Op::Interpolation(interp) => {
                emit_gen_interp(cx, interp, &mut first)?;
            }
            Op::Comment(comment) => {
                start_item(cx, &mut first);
                let _id = cx.walk.mint();
                emit_comment_vnode(cx, comment);
            }
            _ => {
                start_item(cx, &mut first);
                emit_node_child(cx, op)?;
            }
        }
    }
    Ok(())
}

fn emit_gen_interp(
    cx: &mut EmitCx<'_>,
    interp: &InterpolationOp<'_>,
    first: &mut bool,
) -> Result<(), EmitError> {
    let id = cx.walk.mint();
    match interp.expression {
        ExprRef::Js(js) => {
            start_item(cx, first);
            emit_js_to_display_string(cx, js);
            Ok(())
        }
        ExprRef::Opaque(opaque) if opaque.reason == OpaqueReason::Compound => {
            let id = id.ok_or(EmitError::unsupported_at(
                Reason::WalkIdOverflow,
                interp.span,
            ))?;
            let parts = cx
                .facts
                .text_facts
                .get(id)
                .ok_or(EmitError::unsupported_at_node(
                    Reason::MissingTextFacts,
                    interp.span,
                    id,
                ))?
                .parts
                .clone();
            for part in parts.iter() {
                start_item(cx, first);
                if part.dynamic {
                    emit_dynamic_part(cx, part.text.as_str(), Site::Expression)?;
                } else {
                    emit_plain_text_vnode(cx, part.text.as_str());
                }
            }
            Ok(())
        }
        ExprRef::Opaque(opaque) if is_empty_interpolation(opaque) => {
            start_item(cx, first);
            emit_to_display_string(cx, "");
            Ok(())
        }
        _ => {
            start_item(cx, first);
            emit_raw_interpolation_or_refuse(cx, interp.expression)
        }
    }
}

fn emit_node_child(cx: &mut EmitCx<'_>, op: &Op<'_>) -> Result<(), EmitError> {
    match op {
        Op::Element(element) if is_hoistable(element, cx.is_ts) => {
            emit_hoisted_element(cx, element)
        }
        _ => vnode::emit_array_child(cx, op, false, false),
    }
}

fn emit_inline_child(cx: &mut EmitCx<'_>, op: &Op<'_>) -> Result<(), EmitError> {
    match op {
        Op::Text(_) => emit_create_text_vnode(cx, core::slice::from_ref(op)),
        Op::Interpolation(interp) => {
            let id = cx.walk.mint();
            emit_interpolation(cx, interp, id)
        }
        Op::Comment(comment) => {
            let _id = cx.walk.mint();
            emit_comment_vnode(cx, comment);
            Ok(())
        }
        _ => emit_node_child(cx, op),
    }
}

fn start_item(cx: &mut EmitCx<'_>, first: &mut bool) {
    if !*first {
        cx.buf.push(",");
    }
    *first = false;
    cx.buf.newline();
}
