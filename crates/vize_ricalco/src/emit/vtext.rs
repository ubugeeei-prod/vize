//! `vue.text` realization as the shipped `textContent` prop.

use vize_s2::expr::ExprRef;
use vize_s2::op::VueTextOp;

use super::UnsupportedReason as Reason;
use super::{EmitCx, EmitError};

pub(super) fn admit(text: &VueTextOp<'_>) -> Result<(), EmitError> {
    value(text).map(|_| ())
}

pub(super) fn emit_pair(cx: &mut EmitCx<'_>, text: &VueTextOp<'_>) -> Result<(), EmitError> {
    cx.buf.push("textContent: ");
    super::children::emit_to_display_string(cx, value(text)?.unwrap_or("undefined"));
    Ok(())
}

fn value<'a>(text: &'a VueTextOp<'a>) -> Result<Option<&'a str>, EmitError> {
    match text.value {
        Some(ExprRef::Js(js)) => Ok(Some(js.source)),
        Some(expr) => Err(EmitError::unsupported_at(
            Reason::TextDirectiveExpressionNotJs,
            expr.span(),
        )),
        None => Ok(None),
    }
}
