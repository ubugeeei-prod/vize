//! Text and interpolation children.

mod slot_compound;
mod slot_text;

pub(super) use slot_text::{emit_slot_text_child, emit_slot_text_run};
use vize_davinci::id::NodeId;
use vize_s0::Span;
use vize_s2::expr::{ExprRef, JsExpr, OpaqueReason};
use vize_s2::op::{CommentOp, InterpolationOp, Op, Region};

use super::EmitCx;
use super::EmitError;
use super::UnsupportedReason as Reason;
use super::buf::Buf;
use super::js::escape_js_string;
use super::prefix::Site;
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
            Op::Element(_)
            | Op::Component(_)
            | Op::Comment(_)
            | Op::If(_)
            | Op::For(_)
            | Op::Slot(_) => {
                return Err(EmitError::unsupported_op(
                    Reason::TextRunContainsNonText,
                    op,
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn children_need_text_flag(cx: &EmitCx<'_>, children: &Region<'_>) -> bool {
    let ops = &children.ops;
    if ops.is_empty() {
        return false;
    }
    let mut any_interp = false;
    let mut all_constant = true;
    // The children have not been walked yet, so their ids come from a
    // clone: a text-run op mints exactly one and carries no bindings or
    // children of its own.
    let mut walk = cx.walk.clone();
    for op in ops.iter() {
        let id = walk.mint();
        match op {
            Op::Interpolation(interp) => {
                any_interp = true;
                all_constant &= interpolation_is_constant(cx, interp, id);
            }
            Op::Text(_) => {}
            Op::Comment(_) => return false,
            _ => return false,
        }
    }
    // `is_constant_interpolation`: text made only of `LiteralConst` /
    // `SetupConst` reads never updates, so it carries no TEXT flag. Only
    // an inlined render function ever sees such a bare name — everywhere
    // else the transform has already written `$setup.` in front of it.
    any_interp && !all_constant
}

/// `is_constant_interpolation` over one interpolation op. The shipped
/// lane asks it of every interpolation *child* of the element; S2 folds
/// a text run into a single compound op, so the compound shape answers
/// per dynamic part instead of over the joined source, which never
/// names a binding.
fn interpolation_is_constant(
    cx: &EmitCx<'_>,
    interp: &InterpolationOp<'_>,
    id: Option<NodeId>,
) -> bool {
    if !matches!(
        interp.expression,
        ExprRef::Opaque(opaque) if opaque.reason == OpaqueReason::Compound
    ) {
        return cx.reads_constant_binding(&interp.expression);
    }
    let Some(facts) = id.and_then(|id| cx.facts.text_facts.get(id)) else {
        return false;
    };
    facts
        .parts
        .iter()
        .filter(|part| part.dynamic)
        .all(|part| cx.reads_constant_binding_name(part.text.as_str().trim()))
}

pub(super) fn emit_interpolation(
    cx: &mut EmitCx<'_>,
    interp: &InterpolationOp<'_>,
    id: Option<NodeId>,
) -> Result<(), EmitError> {
    match interp.expression {
        ExprRef::Js(js) if cx.prefixing() => {
            emit_prefixed_to_display_string(cx, js, Site::Expression)
        }
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
            emit_compound_parts(cx, &facts.parts, interp.span, Site::Expression)
        }
        ExprRef::Opaque(opaque) if is_empty_interpolation(opaque) => {
            emit_to_display_string(cx, "");
            Ok(())
        }
        _ if cx.prefixing() => {
            let text = cx.prefixed_expr(&interp.expression, Site::Expression)?;
            emit_to_display_string(cx, text.as_str());
            Ok(())
        }
        _ => emit_raw_interpolation_or_refuse(cx, interp.expression),
    }
}

/// `_toDisplayString(<prefixed>)` — the `prefix_identifiers` spelling of
/// [`emit_js_to_display_string`].
pub(super) fn emit_prefixed_to_display_string(
    cx: &mut EmitCx<'_>,
    js: &JsExpr<'_>,
    site: Site,
) -> Result<(), EmitError> {
    let text = cx.prefixed_js(js, site)?;
    emit_to_display_string(cx, text.as_str());
    Ok(())
}

/// One compound part as the shipped codegen consumed it (a dynamic part
/// is an interpolation the transform prefixed on its own).
pub(super) fn emit_dynamic_part(
    cx: &mut EmitCx<'_>,
    text: &str,
    site: Site,
) -> Result<(), EmitError> {
    if cx.prefixing() {
        let text = cx.prefixed_text(text, site)?;
        emit_to_display_string(cx, text.as_str());
    } else {
        emit_to_display_string(cx, text);
    }
    Ok(())
}

pub(super) fn is_empty_interpolation(expr: &vize_s2::expr::OpaqueExpr<'_>) -> bool {
    expr.reason == OpaqueReason::ParseRejected && expr.source.is_empty()
}

pub(super) fn emit_compound_parts(
    cx: &mut EmitCx<'_>,
    parts: &[TextPart],
    span: Span,
    site: Site,
) -> Result<(), EmitError> {
    if parts.is_empty() {
        return Err(EmitError::unsupported_at(Reason::EmptyCompoundText, span));
    }
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            cx.buf.push(" + ");
        }
        if part.dynamic {
            emit_dynamic_part(cx, part.text.as_str(), site)?;
        } else {
            emit_quoted_text(cx, part.text.as_str());
        }
    }
    Ok(())
}

pub(super) fn emit_quoted_text(cx: &mut EmitCx<'_>, content: &str) {
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

pub(super) fn emit_comment_vnode(cx: &mut EmitCx<'_>, comment: &CommentOp<'_>) {
    cx.buf.use_create_comment();
    cx.buf.push(Buf::create_comment_alias());
    cx.buf.push("(\"");
    cx.buf.push(escape_js_string(comment.content).as_str());
    cx.buf.push("\")");
}

/// Array-form text run: `_createTextVNode(...)`, matching
/// `codegen/children.rs`'s consecutive-run grouping.
pub(super) fn emit_create_text_vnode(cx: &mut EmitCx<'_>, ops: &[Op<'_>]) -> Result<(), EmitError> {
    if ops.is_empty() {
        return Err(EmitError::unsupported(Reason::EmptyTextRun));
    }
    if emit_once_compound_text_vnodes(cx, ops)? {
        return Ok(());
    }
    let has_interp = ops.iter().any(|op| matches!(op, Op::Interpolation(_)));
    let is_single_space =
        !has_interp && ops.len() == 1 && matches!(&ops[0], Op::Text(text) if text.content == " ");
    cx.buf.use_create_text();
    cx.buf.push(Buf::create_text_alias());
    if is_single_space && cx.once_depth == 0 {
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

fn emit_once_compound_text_vnodes(cx: &mut EmitCx<'_>, ops: &[Op<'_>]) -> Result<bool, EmitError> {
    if cx.once_depth == 0 || cx.once_element_depth != 1 || ops.len() != 1 {
        return Ok(false);
    }
    let Op::Interpolation(interp) = &ops[0] else {
        return Ok(false);
    };
    if !matches!(
        interp.expression,
        ExprRef::Opaque(opaque) if opaque.reason == OpaqueReason::Compound
    ) {
        return Ok(false);
    }
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
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            cx.buf.push(",");
            cx.buf.newline();
        }
        cx.buf.use_create_text();
        cx.buf.push(Buf::create_text_alias());
        cx.buf.push("(");
        if part.dynamic {
            emit_dynamic_part(cx, part.text.as_str(), Site::Expression)?;
            cx.buf.push(", 1 /* TEXT */");
        } else {
            emit_quoted_text(cx, part.text.as_str());
        }
        cx.buf.push(")");
    }
    Ok(true)
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
