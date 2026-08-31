//! JS-string escaping and raw expression helpers copied because this crate
//! cannot depend on `vize_atelier_core`; byte-identical output is the P2-11 bar.

use vize_s0::expression_guard::scan::{
    keyword_allows_regex_after, skip_identifier, skip_line_comment, skip_number, skip_quoted,
    skip_regex,
};
use vize_s0::{Allocator, Span, String, ToCompactString};
use vize_s2::expr::{ExprRef, JsExpr, OpaqueExpr, OpaqueReason};

fn decode_html_entities(s: &str) -> String {
    if !s.contains('&') {
        return String::from(s);
    }
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '&' && chars.peek() == Some(&'#') {
            chars.next();
            let is_hex = chars.peek() == Some(&'x') || chars.peek() == Some(&'X');
            if is_hex {
                chars.next();
            }
            let mut num_str = String::default();
            while let Some(&ch) = chars.peek() {
                if ch == ';' {
                    chars.next();
                    break;
                }
                let is_valid_char =
                    (is_hex && ch.is_ascii_hexdigit()) || (!is_hex && ch.is_ascii_digit());
                if is_valid_char {
                    num_str.push(ch);
                    chars.next();
                } else {
                    break;
                }
            }
            if !num_str.is_empty() {
                let codepoint = if is_hex {
                    u32::from_str_radix(num_str.as_str(), 16).ok()
                } else {
                    num_str.as_str().parse::<u32>().ok()
                };
                if let Some(cp) = codepoint
                    && let Some(decoded_char) = char::from_u32(cp)
                {
                    result.push(decoded_char);
                    continue;
                }
            }
            result.push('&');
            result.push('#');
            if is_hex {
                result.push('x');
            }
            result.push_str(num_str.as_str());
        } else {
            result.push(c);
        }
    }
    result
}

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

pub(super) enum RawJs<'a> {
    Borrowed(&'a str),
    Owned(String),
}

impl RawJs<'_> {
    pub(super) fn as_str(&self) -> &str {
        match self {
            Self::Borrowed(source) => source,
            Self::Owned(source) => source.as_str(),
        }
    }
}

pub(super) fn js_expr_source<'a>(js: &JsExpr<'a>) -> RawJs<'a> {
    line_comment_source_as_block(js.source, js.span.start)
        .unwrap_or_else(|| RawJs::Borrowed(js.source))
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
        || line_comment_value_as_js(opaque).is_some()
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

fn source_is_js(source: &str, span_start: u32) -> bool {
    let allocator = Allocator::new();
    JsExpr::parse_in(
        &allocator,
        source,
        Span::new(span_start, span_start + source.len() as u32),
    )
    .is_ok()
}

fn trailing_block_comment_value_is_js(source: &str, span: Span) -> bool {
    strip_trailing_block_comments(source)
        .map(|stripped| source_is_js(stripped, span.start))
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

fn line_comment_value_as_js<'a>(opaque: &OpaqueExpr<'a>) -> Option<RawJs<'a>> {
    line_comment_source_as_block(opaque.source, opaque.span.start)
}

fn line_comment_source_as_block<'a>(source: &'a str, span_start: u32) -> Option<RawJs<'a>> {
    if !source.contains("//") {
        return None;
    }
    let converted = convert_line_comments_to_block(source);
    if converted == source || !source_is_js(converted.as_str(), span_start) {
        return None;
    }
    Some(RawJs::Owned(converted))
}

fn convert_line_comments_to_block(content: &str) -> String {
    let bytes = content.as_bytes();
    let mut result = String::with_capacity(content.len());
    let mut can_start_regex = true;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\'' | b'"' | b'`' => {
                let end = skip_quoted(bytes, i + 1, bytes[i]).min(bytes.len());
                result.push_str(&content[i..end]);
                i = end;
                can_start_regex = false;
            }
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                let start = i + 2;
                let end = skip_line_comment(bytes, start);
                result.push_str("/* ");
                result.push_str(content[start..end].trim_end().replace("*/", "* /").as_str());
                result.push_str(" */");
                i = end;
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                let mut end = i + 2;
                while end + 1 < bytes.len() && !(bytes[end] == b'*' && bytes[end + 1] == b'/') {
                    end += 1;
                }
                let end = if end + 1 < bytes.len() {
                    end + 2
                } else {
                    bytes.len()
                };
                result.push_str(&content[i..end]);
                i = end;
            }
            b'/' if can_start_regex => {
                if let Some(end) = skip_regex(bytes, i + 1) {
                    result.push_str(&content[i..end]);
                    i = end;
                    can_start_regex = false;
                } else {
                    result.push('/');
                    i += 1;
                }
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'$' => {
                let end = skip_identifier(bytes, i + 1);
                result.push_str(&content[i..end]);
                can_start_regex = keyword_allows_regex_after(&bytes[i..end]);
                i = end;
            }
            b'0'..=b'9' => {
                let end = skip_number(bytes, i + 1);
                result.push_str(&content[i..end]);
                i = end;
                can_start_regex = false;
            }
            b')' | b']' | b'}' => {
                result.push(bytes[i] as char);
                i += 1;
                can_start_regex = false;
            }
            b'+' | b'-' if bytes.get(i + 1) == Some(&bytes[i]) => {
                result.push_str(&content[i..i + 2]);
                i += 2;
                can_start_regex = false;
            }
            _ => {
                let ch = content[i..].chars().next().unwrap_or('\u{FFFD}');
                result.push(ch);
                i += ch.len_utf8();
                if !ch.is_ascii_whitespace() {
                    can_start_regex = true;
                }
            }
        }
    }
    result
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
