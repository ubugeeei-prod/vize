//! `vue.html` realization as the shipped `innerHTML` prop.

use vize_s2::op::VueHtmlOp;

use super::UnsupportedReason as Reason;
use super::js::{RawJs, expr_source};
use super::{EmitCx, EmitError};

pub(super) fn admit(html: &VueHtmlOp<'_>) -> Result<(), EmitError> {
    value(html).map(|_| ())
}

pub(super) fn emit_pair(cx: &mut EmitCx<'_>, html: &VueHtmlOp<'_>) -> Result<(), EmitError> {
    cx.buf.push("innerHTML: ");
    match value(html)? {
        Some(source) => cx.buf.push(source.as_str()),
        None => cx.buf.push("undefined"),
    }
    Ok(())
}

fn value<'a>(html: &'a VueHtmlOp<'a>) -> Result<Option<RawJs<'a>>, EmitError> {
    match html.value {
        Some(expr) => expr_source(&expr, false)
            .map(Some)
            .ok_or_else(|| EmitError::unsupported_at(Reason::HtmlExpressionNotJs, expr.span())),
        None => Ok(None),
    }
}
