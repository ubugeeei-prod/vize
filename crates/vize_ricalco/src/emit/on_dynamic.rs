//! Dynamic-name `ui.on` (`@[event]`) emission.

use vize_s2::expr::{ExprRef, JsExpr};
use vize_s2::op::{DynamicName, OnOp};

use super::buf::Buf;
use super::js::is_valid_js_identifier;
use super::{EmitCx, EmitError, UnsupportedReason as Reason};

pub(super) fn is_dynamic_on_name(on: &OnOp<'_>) -> bool {
    matches!(on.name, Some(DynamicName::Dynamic(_)))
}

pub(super) fn admit(on: &OnOp<'_>) -> Result<(), EmitError> {
    dynamic_name(on)?;
    match on.handler {
        None | Some(ExprRef::Js(_)) => Ok(()),
        Some(expr) => Err(EmitError::unsupported_at(
            Reason::OnHandlerNotJs,
            expr.span(),
        )),
    }
}

pub(super) fn emit_pair(cx: &mut EmitCx<'_>, on: &OnOp<'_>) -> Result<(), EmitError> {
    let js = dynamic_name(on)?;
    cx.buf.use_to_handler_key();
    cx.buf.push("[");
    cx.buf.push(Buf::to_handler_key_alias());
    cx.buf.push("(");
    emit_key_source(cx, js.source);
    cx.buf.push(")]: ");
    emit_value(cx, on)
}

pub(super) fn emit_value(cx: &mut EmitCx<'_>, on: &OnOp<'_>) -> Result<(), EmitError> {
    let classified = super::on::classify_dynamic_modifiers(on.modifiers.iter().copied());
    super::on::emit_wrapped_handler(cx, on, &classified)
}

pub(super) fn forces_inline(on: &OnOp<'_>) -> bool {
    on.modifiers
        .iter()
        .any(|modifier| !matches!(*modifier, "capture" | "once" | "passive"))
}

fn dynamic_name<'a>(on: &'a OnOp<'a>) -> Result<&'a JsExpr<'a>, EmitError> {
    match on.name {
        Some(DynamicName::Dynamic(ExprRef::Js(js))) => Ok(js),
        Some(DynamicName::Dynamic(expr)) => {
            Err(EmitError::unsupported_at(Reason::OnNameNotJs, expr.span()))
        }
        Some(DynamicName::Static(_)) | None => {
            Err(EmitError::unsupported_at(Reason::OnNameNotStatic, on.span))
        }
    }
}

pub(super) fn emit_key_source(cx: &mut EmitCx<'_>, source: &str) {
    if let Some(local) = source.strip_prefix("_ctx.")
        && cx.is_scope_name(local)
    {
        cx.buf.push(local);
        return;
    }
    if cx.is_scope_name(source) || source.contains('.') || source.starts_with('_') {
        cx.buf.push(source);
        return;
    }
    if is_valid_js_identifier(source) || (!source.starts_with('$') && !source.starts_with('`')) {
        cx.buf.push("_ctx.");
    }
    cx.buf.push(source);
}
