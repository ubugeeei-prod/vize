//! `ui.bind` value admission for emitter-only JS edge cases.

use vize_s0::Span;
use vize_s2::expr::{ExprRef, JsExpr};
use vize_s2::op::BindOp;

use super::entity::decode_html_entities;
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
            Self::Js(js) => cx.buf.push(bind_js_source(js).as_str()),
            Self::RawJs(source) => cx.buf.push(source.as_str()),
        }
    }

    pub(super) fn emit_authored(&self, cx: &mut EmitCx<'_>, bind: &BindOp<'_>) {
        let Self::Js(js) = self else {
            self.emit(cx);
            return;
        };
        let raw_source = js_expr_source(js);
        let decoded = raw_source
            .as_str()
            .contains('&')
            .then(|| decode_html_entities(raw_source.as_str()));
        let source = decoded.as_deref().unwrap_or_else(|| raw_source.as_str());
        let source_root = cx.source;
        if let Some((leading, trailing)) =
            authored_value_padding(source_root, bind, raw_source.as_str(), js.span)
        {
            cx.buf.push(leading);
            cx.buf.push(source);
            cx.buf.push(trailing);
        } else {
            cx.buf.push(source);
        }
    }

    pub(super) const fn js(&self) -> Option<&JsExpr<'_>> {
        match self {
            Self::Js(js) => Some(js),
            Self::RawJs(_) => None,
        }
    }
}

pub(super) fn bind_js_source<'a>(js: &JsExpr<'a>) -> RawJs<'a> {
    let source = js_expr_source(js);
    if source.as_str().contains('&') {
        RawJs::Owned(decode_html_entities(source.as_str()))
    } else {
        source
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

pub(super) fn authored_value_padding<'a>(
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
        return authored_quoted_value_padding(source, attr_start, attr_end);
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

fn authored_quoted_value_padding(
    source: &str,
    attr_start: usize,
    attr_end: usize,
) -> Option<(&str, &str)> {
    if attr_start > attr_end || attr_end > source.len() {
        return None;
    }
    let attr = source.get(attr_start..attr_end)?;
    let quote_pos = attr
        .as_bytes()
        .iter()
        .position(|byte| matches!(*byte, b'\'' | b'"'))?;
    let quote = attr.as_bytes()[quote_pos];
    let inner_start = quote_pos + 1;
    let close_rel = attr
        .get(inner_start..)?
        .as_bytes()
        .iter()
        .position(|byte| *byte == quote)?;
    let inner_end = inner_start + close_rel;
    let inner = attr.get(inner_start..inner_end)?;
    let leading_len = inner
        .bytes()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(inner.len());
    let trailing_start = inner
        .bytes()
        .rposition(|byte| !byte.is_ascii_whitespace())
        .map_or(inner.len(), |index| index + 1);
    let leading = inner.get(..leading_len)?;
    let trailing = inner.get(trailing_start..)?;
    if leading.is_empty() && trailing.is_empty() {
        return None;
    }
    Some((leading, trailing))
}
