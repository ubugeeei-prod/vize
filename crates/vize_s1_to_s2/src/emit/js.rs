//! JS-string escaping and raw expression helpers copied because this crate
//! cannot depend on `vize_atelier_core`; byte-identical output is the P2-11 bar.

use vize_s0::{Span, String, ToCompactString};
use vize_s2::expr::{ExprRef, JsExpr, OpaqueExpr, OpaqueReason};

use super::entity::decode_html_entities;
pub(super) use super::js_comment::RawJs;
use super::js_comment::line_comment_source_as_block;

#[inline]
fn byte_may_need_js_escaping(b: u8) -> bool {
    b < 0x20 || b == b'"' || b == b'&' || b == b'\\' || b == 0x7F || b == 0xC2
}

fn push_hex4(out: &mut String, value: u32) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    out.push_str("\\u");
    out.push(HEX[((value >> 12) & 0xF) as usize] as char);
    out.push(HEX[((value >> 8) & 0xF) as usize] as char);
    out.push(HEX[((value >> 4) & 0xF) as usize] as char);
    out.push(HEX[(value & 0xF) as usize] as char);
}

/// Same rule as `vize_atelier_core::codegen::helpers::is_valid_js_identifier`.
pub(super) fn is_valid_js_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

pub(super) fn js_expr_source<'a>(js: &JsExpr<'a>) -> RawJs<'a> {
    line_comment_source_as_block(js.source, js.span.start)
        .unwrap_or_else(|| RawJs::Borrowed(js.source))
}

pub(super) fn push_js_expr(cx: &mut super::EmitCx<'_>, js: &JsExpr<'_>) {
    let source = js_expr_source(js);
    cx.buf.push(source.as_str());
}

pub(super) fn expr_source<'a>(expr: &ExprRef<'a>, allow_identifier: bool) -> Option<RawJs<'a>> {
    match expr {
        ExprRef::Js(js) => Some(js_expr_source(js)),
        ExprRef::Opaque(opaque) if opaque.reason == OpaqueReason::MultiStatement => {
            line_comment_source_as_block(opaque.source, opaque.span.start)
        }
        _ => parse_rejected_raw_js(expr, allow_identifier),
    }
}

pub(super) fn parse_rejected_raw_js<'a>(
    expr: &ExprRef<'a>,
    allow_identifier: bool,
) -> Option<RawJs<'a>> {
    let opaque = parse_rejected_opaque(expr)?;
    if allow_identifier && is_valid_js_identifier(opaque.source) {
        return Some(RawJs::Borrowed(opaque.source));
    }
    if trailing_block_comment_value_is_js(opaque.source, opaque.span) {
        return Some(RawJs::Borrowed(opaque.source));
    }
    line_comment_source_as_block(opaque.source, opaque.span.start)
}

pub(super) fn parse_rejected_original_raw_js<'a>(
    expr: &ExprRef<'a>,
    allow_identifier: bool,
) -> Option<&'a str> {
    let opaque = parse_rejected_opaque(expr)?;
    if allow_identifier && is_valid_js_identifier(opaque.source) {
        return Some(opaque.source);
    }
    if trailing_block_comment_value_is_js(opaque.source, opaque.span)
        || line_comment_source_as_block(opaque.source, opaque.span.start).is_some()
    {
        return Some(opaque.source);
    }
    None
}

fn parse_rejected_opaque<'a>(expr: &ExprRef<'a>) -> Option<&'a OpaqueExpr<'a>> {
    let ExprRef::Opaque(opaque) = expr else {
        return None;
    };
    (opaque.reason == OpaqueReason::ParseRejected).then_some(opaque)
}

fn trailing_block_comment_value_is_js(source: &str, span: Span) -> bool {
    strip_trailing_block_comments(source)
        .map(|stripped| super::js_comment::source_is_js(stripped, span.start))
        .unwrap_or(false)
}

fn strip_trailing_block_comments(mut source: &str) -> Option<&str> {
    let original = source;
    loop {
        source = source.trim_end();
        if let Some(end) = source.strip_suffix("*/")
            && let Some(start) = end.rfind("/*")
        {
            source = &end[..start];
            continue;
        }
        break;
    }
    let stripped = source.trim_end();
    (stripped.len() < original.len() && !stripped.trim().is_empty()).then_some(stripped)
}

/// Escape `s` for a JavaScript double-quoted string literal, matching
/// `vize_atelier_core::codegen::helpers::escape_js_string`.
pub(super) fn escape_js_string(s: &str) -> String {
    if !s.bytes().any(byte_may_need_js_escaping) {
        return String::from(s);
    }
    let decoded = decode_html_entities(s);
    let mut result = String::with_capacity(decoded.len());
    for c in decoded.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\x08' => result.push_str("\\b"),
            '\x0C' => result.push_str("\\f"),
            c if c.is_control() => push_hex4(&mut result, c as u32),
            c => result.push(c),
        }
    }
    result
}

/// Mirror Vue's `toValidAssetId` (compiler-core utils, issue #4422):
/// word characters pass through, `-` becomes `_`, every other character
/// is replaced by its char code as a decimal string.
pub(crate) fn asset_ident(kind: &str, name: &str) -> String {
    let mut ident = String::from("_");
    ident.push_str(kind);
    ident.push('_');
    for c in name.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            ident.push(c);
        } else if c == '-' {
            ident.push('_');
        } else {
            ident.push_str((c as u32).to_compact_string().as_str());
        }
    }
    ident
}

pub(super) fn push_ident_key(cx: &mut super::EmitCx<'_>, name: &str) {
    if !is_valid_js_identifier(name) {
        cx.buf.push("\"");
        cx.buf.push(name);
        cx.buf.push("\"");
    } else {
        cx.buf.push(name);
    }
}

#[cfg(test)]
mod tests;
