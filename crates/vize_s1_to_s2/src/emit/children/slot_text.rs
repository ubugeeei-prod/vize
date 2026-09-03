use vize_davinci::id::NodeId;
use vize_s2::expr::{ExprRef, OpaqueReason};
use vize_s2::op::{InterpolationOp, Op};

use super::super::buf::Buf;
use super::super::js::escape_js_string;
use super::super::prefix::Site;
use super::super::{EmitCx, EmitError, UnsupportedReason as Reason};
use super::{
    emit_compound_parts, emit_create_text_vnode, emit_quoted_text, emit_to_display_string,
    is_empty_interpolation, slot_compound::emit_slot_compound_parts,
};

/// Slot default children emit one or more `_createTextVNode`s.
pub(in crate::emit) fn emit_slot_text_child(
    cx: &mut EmitCx<'_>,
    op: &Op<'_>,
) -> Result<(), EmitError> {
    let Op::Interpolation(interp) = op else {
        if let Op::Text(text) = op {
            emit_slot_plain_text_vnode(cx, text.content);
            return Ok(());
        }
        return emit_create_text_vnode(cx, core::slice::from_ref(op));
    };
    match interp.expression {
        ExprRef::Js(_) if cx.prefixing() => {
            let text = cx.prefixed_expr(&interp.expression, Site::SlotText)?;
            emit_slot_raw_interpolation(cx, text.as_str())
        }
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
        _ if cx.prefixing() => {
            let text = cx.prefixed_expr(&interp.expression, Site::SlotText)?;
            emit_slot_raw_interpolation(cx, text.as_str())
        }
        _ => {
            if let Some(source) =
                super::super::js::parse_rejected_original_raw_js(&interp.expression, false)
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

pub(in crate::emit) fn emit_slot_text_run(
    cx: &mut EmitCx<'_>,
    ops: &[Op<'_>],
) -> Result<(), EmitError> {
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
        ExprRef::Js(_) if cx.prefixing() => {
            let text = cx.prefixed_expr(&interp.expression, Site::SlotText)?;
            emit_to_display_string(cx, text.as_str());
            Ok(())
        }
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
            emit_compound_parts(cx, &facts.parts, interp.span, Site::SlotText)
        }
        ExprRef::Opaque(opaque) if is_empty_interpolation(opaque) => {
            emit_to_display_string(cx, "");
            Ok(())
        }
        _ if cx.prefixing() => {
            let text = cx.prefixed_expr(&interp.expression, Site::SlotText)?;
            emit_to_display_string(cx, text.as_str());
            Ok(())
        }
        _ => {
            if let Some(source) =
                super::super::js::parse_rejected_original_raw_js(&interp.expression, false)
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
