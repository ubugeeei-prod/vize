//! `vue.html` realization as the shipped `innerHTML` prop.

use vize_s2::op::VueHtmlOp;

use super::UnsupportedReason as Reason;
use super::entity::decode_html_entities;
use super::js::{RawJs, expr_source};
use super::{EmitCx, EmitError};
use vize_s0::Span;

pub(super) fn admit(html: &VueHtmlOp<'_>) -> Result<(), EmitError> {
    value(html).map(|_| ())
}

pub(super) fn emit_pair(cx: &mut EmitCx<'_>, html: &VueHtmlOp<'_>) -> Result<(), EmitError> {
    cx.buf.push("innerHTML: ");
    match html.value {
        Some(expr) if cx.prefixing() => {
            value(html)?;
            let text = cx.prefixed_bind_expr(&expr)?;
            cx.buf.push(text.as_str());
        }
        Some(expr) => {
            let raw_source = expr_source(&expr, false).ok_or_else(|| {
                EmitError::unsupported_at(Reason::HtmlExpressionNotJs, expr.span())
            })?;
            let decoded = raw_source
                .as_str()
                .contains('&')
                .then(|| decode_html_entities(raw_source.as_str()));
            let source = decoded.as_deref().unwrap_or_else(|| raw_source.as_str());
            if let Some((leading, trailing)) =
                authored_html_padding(cx.source, html.span, raw_source.as_str(), expr.span())
            {
                cx.buf.push(leading);
                cx.buf.push(source);
                cx.buf.push(trailing);
            } else {
                cx.buf.push(source);
            }
        }
        None => cx.buf.push("undefined"),
    }
    Ok(())
}

fn authored_html_padding<'a>(
    source: &'a str,
    html_span: Span,
    value: &str,
    value_span: Span,
) -> Option<(&'a str, &'a str)> {
    let attr_start = usize::try_from(html_span.start).ok()?;
    let attr_end = usize::try_from(html_span.end).ok()?;
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

fn value<'a>(html: &'a VueHtmlOp<'a>) -> Result<Option<RawJs<'a>>, EmitError> {
    match html.value {
        Some(expr) => expr_source(&expr, false)
            .map(Some)
            .ok_or_else(|| EmitError::unsupported_at(Reason::HtmlExpressionNotJs, expr.span())),
        None => Ok(None),
    }
}
