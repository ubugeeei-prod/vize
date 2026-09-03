//! Runtime-directive entry pieces: authored value padding, `v-show`
//! entries and dynamic arguments.

use vize_s2::op::{DynamicName, VueShowOp};

use super::super::buf::Buf;
use super::super::prefix::Site;
use super::super::{EmitCx, EmitError};

pub(super) fn authored_value_padding<'a>(
    source: &'a str,
    owner_span: vize_s0::Span,
    value: &str,
    value_span: vize_s0::Span,
) -> Option<(&'a str, &'a str)> {
    let attr_start = usize::try_from(owner_span.start).ok()?;
    let attr_end = usize::try_from(owner_span.end).ok()?;
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

pub(super) fn emit_show_entry(cx: &mut EmitCx<'_>, show: &VueShowOp<'_>) -> Result<(), EmitError> {
    cx.buf.use_v_show();
    cx.buf.push("  [");
    cx.buf.push(Buf::v_show_alias());
    cx.buf.push(", ");
    let source = super::show_value(show)?;
    if cx.prefixing() {
        cx.push_prefixed_expr(&show.value, Site::Expression)?;
    } else if let Some((leading, trailing)) =
        authored_value_padding(cx.source, show.span, source.as_str(), show.value.span())
    {
        cx.buf.push(leading);
        cx.buf.push(source.as_str());
        cx.buf.push(trailing);
    } else {
        cx.buf.push(source.as_str());
    }
    cx.buf.push("]");
    Ok(())
}

pub(super) fn emit_argument(
    cx: &mut EmitCx<'_>,
    argument: DynamicName<'_>,
) -> Result<(), EmitError> {
    match argument {
        DynamicName::Static(name) => {
            cx.buf.push("\"");
            cx.buf.push(name);
            cx.buf.push("\"");
            Ok(())
        }
        DynamicName::Dynamic(expr) => {
            let source = super::js_expr(expr)?;
            if cx.prefixing() {
                return cx.push_prefixed_expr(&expr, Site::Raw);
            }
            cx.buf.push(source.as_str());
            Ok(())
        }
    }
}
