//! `vue.text` realization as the shipped `textContent` prop.

use vize_s2::op::VueTextOp;

use super::UnsupportedReason as Reason;
use super::js::{RawJs, expr_source};
use super::{EmitCx, EmitError};

pub(super) fn admit(text: &VueTextOp<'_>) -> Result<(), EmitError> {
    value(text).map(|_| ())
}

pub(super) fn emit_pair(cx: &mut EmitCx<'_>, text: &VueTextOp<'_>) -> Result<(), EmitError> {
    cx.buf.push("textContent: ");
    match value(text)? {
        Some(source) => super::children::emit_to_display_string(cx, source.as_str()),
        None => super::children::emit_to_display_string(cx, "undefined"),
    }
    Ok(())
}

fn value<'a>(text: &'a VueTextOp<'a>) -> Result<Option<RawJs<'a>>, EmitError> {
    match text.value {
        Some(expr) => expr_source(&expr, false).map(Some).ok_or_else(|| {
            EmitError::unsupported_at(Reason::TextDirectiveExpressionNotJs, expr.span())
        }),
        None => Ok(None),
    }
}
