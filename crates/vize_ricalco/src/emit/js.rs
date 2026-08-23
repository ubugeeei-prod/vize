//! The shipped codegen's JS-string escape, copied because this crate
//! cannot depend on `vize_atelier_core` (published, `std`; ricalco stays
//! `no_std + alloc`). Byte-identical output is the P2-11 bar.

use vize_carton::String;

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
