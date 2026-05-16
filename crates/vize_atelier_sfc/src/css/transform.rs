//! V-bind extraction and byte-level utility functions.
//!
//! Handles extracting `v-bind()` expressions from CSS and transforming them
//! into CSS custom properties (variables). Also provides low-level byte search
//! utilities used by both this module and the scoped CSS module.

use vize_carton::{Bump, BumpVec, String, ToCompactString};

/// Extract v-bind() expressions and transform them to CSS variables
pub(crate) fn extract_and_transform_v_bind<'a>(
    bump: &'a Bump,
    css: &str,
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
                let expr_bytes = &css_bytes[start..start + end];
                let expr_str = unsafe { std::str::from_utf8_unchecked(expr_bytes) }.trim();
                let expr_str = expr_str.trim_matches(|c| c == '"' || c == '\'');
                vars.push(expr_str.to_compact_string());

                // Generate hash and write var(--hash-expr)
                result.extend_from_slice(b"var(--");
                write_v_bind_hash(&mut result, expr_str);
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

    // SAFETY: input is valid UTF-8, we only add ASCII bytes
    let result_str = unsafe { std::str::from_utf8_unchecked(bump.alloc_slice_copy(&result)) };
    (result_str, vars)
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
