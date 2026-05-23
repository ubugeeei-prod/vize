//! V-bind extraction and byte-level utility functions.
//!
//! Handles extracting `v-bind()` expressions from CSS and transforming them
//! into CSS custom properties (variables). Also provides low-level byte search
//! utilities used by both this module and the scoped CSS module.

use vize_carton::{Bump, BumpVec, String, ToCompactString};

/// Extract v-bind() expressions and transform them to CSS variables
#[cfg(test)]
pub(crate) fn extract_and_transform_v_bind<'a>(
    bump: &'a Bump,
    css: &str,
) -> (&'a str, Vec<String>) {
    extract_and_transform_v_bind_with_scope(bump, css, None)
}

/// Extract v-bind() expressions and transform them using a Vue SFC scope id.
pub(crate) fn extract_and_transform_v_bind_with_scope<'a>(
    bump: &'a Bump,
    css: &str,
    scope_id: Option<&str>,
) -> (&'a str, Vec<String>) {
    let css_bytes = css.as_bytes();
    let mut vars = Vec::new();
    let mut result = BumpVec::with_capacity_in(css_bytes.len() * 2, bump);
    let mut pos = 0;

    while pos < css_bytes.len() {
        if let Some(rel_pos) = find_bytes(&css_bytes[pos..], b"v-bind(") {
            let actual_pos = pos + rel_pos;
            let start = actual_pos + 7;

            if let Some(end) = find_byte(&css_bytes[start..], b')') {
                // Copy everything before v-bind(
                result.extend_from_slice(&css_bytes[pos..actual_pos]);

                // Extract expression
                let Some(expr_str) = css.get(start..start + end).map(str::trim) else {
                    pos = start + end + 1;
                    result.extend_from_slice(&css_bytes[actual_pos..pos]);
                    continue;
                };
                let expr_str = trim_outer_quotes(expr_str);
                vars.push(expr_str.to_compact_string());

                // Generate CSS custom property reference.
                result.extend_from_slice(b"var(--");
                if let Some(scope_id) = scope_id {
                    result.extend_from_slice(scoped_v_bind_name(scope_id, expr_str).as_bytes());
                } else {
                    write_v_bind_hash(&mut result, expr_str);
                }
                result.push(b')');

                pos = start + end + 1;
            } else {
                result.extend_from_slice(&css_bytes[pos..]);
                break;
            }
        } else {
            result.extend_from_slice(&css_bytes[pos..]);
            break;
        }
    }

    // SAFETY: `result` is assembled from byte slices borrowed from `css` plus
    // ASCII-only delimiters, hashes, and sanitized v-bind suffixes. Every copied
    // slice boundary comes from `str::find`/byte scans over ASCII tokens, so it
    // never cuts through a UTF-8 code point. The bump copy keeps the returned
    // `&str` alive for the caller's arena lifetime while avoiding a second UTF-8
    // validation pass on this hot CSS transform path.
    let result_str = unsafe { std::str::from_utf8_unchecked(bump.alloc_slice_copy(&result)) };
    (result_str, vars)
}

pub(crate) fn trim_outer_quotes(expr: &str) -> &str {
    let bytes = expr.as_bytes();
    if bytes.len() >= 2
        && matches!(bytes.first(), Some(b'"' | b'\''))
        && bytes.first() == bytes.last()
    {
        &expr[1..expr.len() - 1]
    } else {
        expr
    }
}

/// Generate the Vue-compatible CSS variable name for a scoped SFC v-bind().
pub(crate) fn scoped_v_bind_name(scope_id: &str, expr: &str) -> String {
    let scope_id = scope_id.strip_prefix("data-v-").unwrap_or(scope_id);
    let mut result = String::with_capacity(scope_id.len() + expr.len() + 1);
    result.push_str(scope_id);
    result.push('-');
    write_escaped_css_var_suffix(&mut result, expr);
    result
}

fn write_escaped_css_var_suffix(out: &mut String, expr: &str) {
    for c in expr.chars() {
        if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
            out.push(c);
        } else {
            out.push('\\');
            out.push(c);
        }
    }
}

/// Write v-bind variable hash to output
fn write_v_bind_hash(out: &mut BumpVec<u8>, expr: &str) {
    let hash: u32 = expr
        .bytes()
        .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));

    // Write hash as hex
    write_hex_u32(out, hash);
    out.push(b'-');

    // Write sanitized expression
    for b in expr.bytes() {
        match b {
            b'.' | b'[' | b']' | b'(' | b')' => out.push(b'_'),
            _ => out.push(b),
        }
    }
}

/// Write u32 as 8-digit hex
fn write_hex_u32(out: &mut BumpVec<u8>, val: u32) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    out.push(HEX[((val >> 28) & 0xF) as usize]);
    out.push(HEX[((val >> 24) & 0xF) as usize]);
    out.push(HEX[((val >> 20) & 0xF) as usize]);
    out.push(HEX[((val >> 16) & 0xF) as usize]);
    out.push(HEX[((val >> 12) & 0xF) as usize]);
    out.push(HEX[((val >> 8) & 0xF) as usize]);
    out.push(HEX[((val >> 4) & 0xF) as usize]);
    out.push(HEX[(val & 0xF) as usize]);
}

/// Find byte sequence in slice
#[inline]
pub(crate) fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

/// Find single byte in slice
#[inline]
pub(crate) fn find_byte(haystack: &[u8], needle: u8) -> Option<usize> {
    haystack.iter().position(|&b| b == needle)
}

/// Reverse find single byte in slice
#[inline]
pub(crate) fn rfind_byte(haystack: &[u8], needle: u8) -> Option<usize> {
    haystack.iter().rposition(|&b| b == needle)
}

/// Find the matching closing parenthesis
pub(crate) fn find_matching_paren(s: &str) -> Option<usize> {
    let mut depth = 1u32;
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}
