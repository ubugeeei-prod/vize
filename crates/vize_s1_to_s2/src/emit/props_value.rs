//! `ui.bind` value admission for emitter-only JS edge cases.

use vize_s0::Span;
use vize_s2::expr::{ExprRef, JsExpr};
use vize_s2::op::BindOp;

use super::js::{RawJs, js_expr_source};
use super::props_bind;
use super::{EmitCx, EmitError};

pub(super) enum BindValue<'a> {
    Js(&'a JsExpr<'a>),
    RawJs(RawJs<'a>),
}

impl BindValue<'_> {
    pub(super) fn emit(&self, cx: &mut EmitCx<'_>) {
        match self {
            Self::Js(js) => cx.buf.push(js_expr_source(js).as_str()),
            Self::RawJs(source) => cx.buf.push(source.as_str()),
        }
    }

    pub(super) fn emit_authored(&self, cx: &mut EmitCx<'_>, bind: &BindOp<'_>) {
        let Self::Js(js) = self else {
            self.emit(cx);
            return;
        };
        let source = js_expr_source(js);
        let source_root = cx.source;
        if let Some((leading, trailing)) =
            authored_value_padding(source_root, bind, source.as_str(), js.span)
        {
            cx.buf.push(leading);
            cx.buf.push(source.as_str());
            cx.buf.push(trailing);
        } else {
            cx.buf.push(source.as_str());
        }
    }

    pub(super) const fn js(&self) -> Option<&JsExpr<'_>> {
        match self {
            Self::Js(js) => Some(js),
            Self::RawJs(_) => None,
        }
    }
}

pub(super) fn bind_value<'a>(bind: &'a BindOp<'a>) -> Result<BindValue<'a>, EmitError> {
    match bind.value {
        Some(ExprRef::Js(js)) => Ok(BindValue::Js(js)),
        Some(expr) => {
            if let Some(raw) = super::js::parse_rejected_raw_js(&expr, true) {
                Ok(BindValue::RawJs(raw))
            } else {
                props_bind::js_value(bind).map(BindValue::Js)
            }
        }
        None => props_bind::js_value(bind).map(BindValue::Js),
    }
}

fn authored_value_padding<'a>(
    source: &'a str,
    bind: &BindOp<'_>,
    value: &str,
    value_span: Span,
) -> Option<(&'a str, &'a str)> {
    let attr_start = usize::try_from(bind.span.start).ok()?;
    let attr_end = usize::try_from(bind.span.end).ok()?;
    let value_start = usize::try_from(value_span.start).ok()?;
    let value_end = usize::try_from(value_span.end).ok()?;
    if attr_start > value_start
        || value_start > value_end
        || value_end > attr_end
        || attr_end > source.len()
        || source.get(value_start..value_end)? != value
    {
        return None;
    }
    let before = source.get(attr_start..value_start)?;
    let quote_pos = before
        .as_bytes()
        .iter()
        .rposition(|byte| matches!(*byte, b'\'' | b'"'))?;
    let quote = before.as_bytes()[quote_pos];
    let leading = before.get(quote_pos + 1..)?;
    let after = source.get(value_end..attr_end)?;
    let trailing_end = after
        .as_bytes()
        .iter()
        .position(|byte| *byte == quote)
        .unwrap_or(after.len());
    let trailing = after.get(..trailing_end)?;
    if leading.is_empty() && trailing.is_empty() {
        return None;
    }
    (leading.bytes().all(|byte| byte.is_ascii_whitespace())
        && trailing.bytes().all(|byte| byte.is_ascii_whitespace()))
    .then_some((leading, trailing))
}
