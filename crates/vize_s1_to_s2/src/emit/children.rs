//! Text and interpolation children.

mod slot_compound;

use slot_compound::emit_slot_compound_parts;
use vize_davinci::id::NodeId;
use vize_s0::Span;
use vize_s2::expr::{ExprRef, JsExpr, OpaqueReason};
use vize_s2::op::{InterpolationOp, Op, Region};

use super::EmitCx;
use super::EmitError;
use super::UnsupportedReason as Reason;
use super::buf::Buf;
use super::js::escape_js_string;
use crate::lower::TextPart;

pub(super) fn emit_text_like(cx: &mut EmitCx<'_>, ops: &[Op<'_>]) -> Result<(), EmitError> {
    for (i, op) in ops.iter().enumerate() {
        let id = cx.walk.mint();
        if i > 0 {
            cx.buf.push(" + ");
        }
        match op {
            Op::Text(text) => emit_quoted_text(cx, text.content),
            Op::Interpolation(interp) => emit_interpolation(cx, interp, id)?,
            Op::Element(_) | Op::Component(_) | Op::If(_) | Op::For(_) | Op::Slot(_) => {
                return Err(EmitError::unsupported_op(
                    Reason::TextRunContainsNonText,
                    op,
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn children_need_text_flag(children: &Region<'_>) -> bool {
    let ops = &children.ops;
    if ops.is_empty() {
        return false;
    }
    let mut any_interp = false;
    for op in ops.iter() {
        match op {
            Op::Interpolation(_) => any_interp = true,
            Op::Text(_) => {}
            _ => return false,
        }
    }
    any_interp
}

pub(super) fn emit_interpolation(
    cx: &mut EmitCx<'_>,
    interp: &InterpolationOp<'_>,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    match interp.expression {
        ExprRef::Js(js) => {
            emit_js_to_display_string(cx, js);
            Ok(())
        }
        ExprRef::Opaque(opaque) if opaque.reason == OpaqueReason::Compound => {
            let id = id.ok_or(EmitError::unsupported_at(
                Reason::WalkIdOverflow,
                interp.span,
            ))?;
            let facts = cx
                .facts
                .text_facts
                .get(id)
                .ok_or(EmitError::unsupported_at_node(
                    Reason::MissingTextFacts,
                    interp.span,
                    id,
                ))?;
            emit_compound_parts(cx, &facts.parts, interp.span)
        }
        ExprRef::Opaque(opaque) if is_empty_interpolation(opaque) => {
            emit_to_display_string(cx, "");
            Ok(())
        }
        _ => emit_raw_interpolation_or_refuse(cx, interp.expression),
    }
}

pub(super) fn is_empty_interpolation(expr: &vize_s2::expr::OpaqueExpr<'_>) -> bool {
    expr.reason == OpaqueReason::ParseRejected && expr.source.is_empty()
}

fn emit_compound_parts(
    cx: &mut EmitCx<'_>,
    parts: &[TextPart],
    span: Span,
) -> Result<(), EmitError> {
    if parts.is_empty() {
        return Err(EmitError::unsupported_at(Reason::EmptyCompoundText, span));
    }
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            cx.buf.push(" + ");
        }
        if part.dynamic {
            emit_to_display_string(cx, part.text.as_str());
        } else {
            emit_quoted_text(cx, part.text.as_str());
        }
    }
    Ok(())
}

fn emit_quoted_text(cx: &mut EmitCx<'_>, content: &str) {
    cx.buf.push("\"");
    cx.buf.push(escape_js_string(content).as_str());
    cx.buf.push("\"");
}

pub(super) fn emit_to_display_string(cx: &mut EmitCx<'_>, source: &str) {
    cx.buf.use_to_display_string();
    cx.buf.push(Buf::to_display_string_alias());
    cx.buf.push("(");
    cx.buf.push(source);
    cx.buf.push(")");
}

pub(super) fn emit_js_to_display_string(cx: &mut EmitCx<'_>, js: &JsExpr<'_>) {
    emit_to_display_string(cx, super::js::js_expr_source(js).as_str());
}

/// Static `_createTextVNode("…")` with no walk mint (compound fallback
/// parts already minted their interpolation op).
pub(super) fn emit_plain_text_vnode(cx: &mut EmitCx<'_>, content: &str) {
    cx.buf.use_create_text();
    cx.buf.push(Buf::create_text_alias());
    if content == " " {
        cx.buf.push("()");
        return;
    }
    cx.buf.push("(\"");
    cx.buf.push(escape_js_string(content).as_str());
    cx.buf.push("\")");
}

/// Array-form text run: `_createTextVNode(...)`, matching
/// `codegen/children.rs`'s consecutive-run grouping.
pub(super) fn emit_create_text_vnode(cx: &mut EmitCx<'_>, ops: &[Op<'_>]) -> Result<(), EmitError> {
    if ops.is_empty() {
        return Err(EmitError::unsupported(Reason::EmptyTextRun));
    }
    let has_interp = ops.iter().any(|op| matches!(op, Op::Interpolation(_)));
    let is_single_space =
        !has_interp && ops.len() == 1 && matches!(&ops[0], Op::Text(text) if text.content == " ");
    cx.buf.use_create_text();
    cx.buf.push(Buf::create_text_alias());
    if is_single_space {
        let _id = cx.walk.mint();
        cx.buf.push("()");
        return Ok(());
    }
    cx.buf.push("(");
    emit_text_like(cx, ops)?;
    if has_interp {
        cx.buf.push(", 1 /* TEXT */");
    }
    cx.buf.push(")");
    Ok(())
}

/// Slot default children emit one or more `_createTextVNode`s.
pub(super) fn emit_slot_text_child(cx: &mut EmitCx<'_>, op: &Op<'_>) -> Result<(), EmitError> {
    let Op::Interpolation(interp) = op else {
        if let Op::Text(text) = op {
            emit_slot_plain_text_vnode(cx, text.content);
            return Ok(());
        }
        return emit_create_text_vnode(cx, core::slice::from_ref(op));
    };
    match interp.expression {
        ExprRef::Js(js) => emit_slot_raw_interpolation(cx, js.source),
        ExprRef::Opaque(opaque) if opaque.reason == OpaqueReason::Compound => {
            let id = cx.walk.mint().ok_or(EmitError::unsupported_at(
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
            emit_slot_compound_parts(cx, &parts, interp.span)
        }
        ExprRef::Opaque(opaque) if is_empty_interpolation(opaque) => {
            emit_create_text_vnode(cx, core::slice::from_ref(op))
        }
        _ => {
            if let Some(source) =
                super::js::parse_rejected_original_raw_js(&interp.expression, false)
            {
                emit_slot_raw_interpolation(cx, source)
            } else {
                Err(EmitError::unsupported_at(
                    Reason::TextExpressionNotEmittable,
                    interp.expression.span(),
                ))
            }
        }
    }
}

pub(super) fn emit_slot_text_run(cx: &mut EmitCx<'_>, ops: &[Op<'_>]) -> Result<(), EmitError> {
    if ops.is_empty() {
        return Err(EmitError::unsupported(Reason::EmptyTextRun));
    }
    let has_interp = ops.iter().any(|op| matches!(op, Op::Interpolation(_)));
    cx.buf.use_create_text();
    cx.buf.push(Buf::create_text_alias());
    cx.buf.push("(");
    emit_slot_text_like(cx, ops)?;
    if has_interp {
        cx.buf.push(", 1 /* TEXT */");
    }
    cx.buf.push(")");
    Ok(())
}

fn emit_slot_plain_text_vnode(cx: &mut EmitCx<'_>, content: &str) {
    let _id = cx.walk.mint();
    cx.buf.use_create_text();
    cx.buf.push(Buf::create_text_alias());
    cx.buf.push("(\"");
    cx.buf.push(escape_js_string(content).as_str());
    cx.buf.push("\")");
}

fn emit_slot_text_like(cx: &mut EmitCx<'_>, ops: &[Op<'_>]) -> Result<(), EmitError> {
    for (i, op) in ops.iter().enumerate() {
        let id = cx.walk.mint();
        if i > 0 {
            cx.buf.push(" + ");
        }
        match op {
            Op::Text(text) => emit_quoted_text(cx, text.content),
            Op::Interpolation(interp) => emit_slot_interpolation(cx, interp, id)?,
            Op::Element(_) | Op::Component(_) | Op::If(_) | Op::For(_) | Op::Slot(_) => {
                return Err(EmitError::unsupported_op(
                    Reason::TextRunContainsNonText,
                    op,
                ));
            }
        }
    }
    Ok(())
}

fn emit_slot_interpolation(
    cx: &mut EmitCx<'_>,
    interp: &InterpolationOp<'_>,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    match interp.expression {
        ExprRef::Js(js) => {
            emit_to_display_string(cx, js.source);
            Ok(())
        }
        ExprRef::Opaque(opaque) if opaque.reason == OpaqueReason::Compound => {
            let id = id.ok_or(EmitError::unsupported_at(
                Reason::WalkIdOverflow,
                interp.span,
            ))?;
            let facts = cx
                .facts
                .text_facts
                .get(id)
                .ok_or(EmitError::unsupported_at_node(
                    Reason::MissingTextFacts,
                    interp.span,
                    id,
                ))?;
            emit_compound_parts(cx, &facts.parts, interp.span)
        }
        ExprRef::Opaque(opaque) if is_empty_interpolation(opaque) => {
            emit_to_display_string(cx, "");
            Ok(())
        }
        _ => {
            if let Some(source) =
                super::js::parse_rejected_original_raw_js(&interp.expression, false)
            {
                emit_to_display_string(cx, source);
                Ok(())
            } else {
                Err(EmitError::unsupported_at(
                    Reason::TextExpressionNotEmittable,
                    interp.expression.span(),
                ))
            }
        }
    }
}

fn emit_slot_raw_interpolation(cx: &mut EmitCx<'_>, source: &str) -> Result<(), EmitError> {
    let _id = cx.walk.mint();
    cx.buf.use_create_text();
    cx.buf.push(Buf::create_text_alias());
    cx.buf.push("(");
    emit_to_display_string(cx, source);
    cx.buf.push(", 1 /* TEXT */)");
    Ok(())
}

pub(super) fn emit_raw_interpolation_or_refuse(
    cx: &mut EmitCx<'_>,
    expression: ExprRef<'_>,
) -> Result<(), EmitError> {
    if let Some(raw) = super::js::parse_rejected_raw_js(&expression, false) {
        emit_to_display_string(cx, raw.as_str());
        Ok(())
    } else {
        Err(EmitError::unsupported_at(
            Reason::TextExpressionNotEmittable,
            expression.span(),
        ))
    }
}
