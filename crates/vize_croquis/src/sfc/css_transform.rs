//! V-bind extraction and byte-level utility functions.
//!
//! Handles extracting `v-bind()` expressions from CSS and transforming them
//! into CSS custom properties (variables). Also provides low-level byte search
//! utilities used by both this module and the scoped CSS module.

mod scanner;

pub use scanner::find_matching_paren;
use scanner::find_next_v_bind;

use vize_carton::{Allocator, String, ToCompactString, Vec as ArenaVec};

/// Extract v-bind() expressions and transform them to CSS variables
#[doc(hidden)]
pub fn extract_and_transform_v_bind<'a>(bump: &'a Allocator, css: &str) -> (&'a str, Vec<String>) {
    extract_and_transform_v_bind_with_scope(bump, css, None)
}

/// Extract v-bind() expressions and transform them using a Vue SFC scope id.
#[doc(hidden)]
pub fn extract_and_transform_v_bind_with_scope<'a>(
    bump: &'a Allocator,
    css: &str,
    scope_id: Option<&str>,
) -> (&'a str, Vec<String>) {
    let css_bytes = css.as_bytes();
    let mut vars = Vec::new();
    let mut result = ArenaVec::with_capacity_in(css_bytes.len() * 2, &bump);
    let mut pos = 0;

    while pos < css_bytes.len() {
        if let Some(actual_pos) = find_next_v_bind(css, pos) {
            let start = actual_pos + 7;

            let Some(after_open) = css.get(start..) else {
                result.extend_from_slice(&css_bytes[pos..]);
                break;
            };

            if let Some(end) = find_matching_paren(after_open) {
                // Copy everything before v-bind(
                result.extend_from_slice(&css_bytes[pos..actual_pos]);

                // Extract expression
                let Some(expr_str) = after_open.get(..end).map(str::trim) else {
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
#[doc(hidden)]
pub fn scoped_v_bind_name(scope_id: &str, expr: &str) -> String {
    let scope_id = scope_id.strip_prefix("data-v-").unwrap_or(scope_id);
    let mut result = String::with_capacity(scope_id.len() + expr.len() + 1);
    result.push_str(scope_id);
    result.push('-');
    write_escaped_css_var_suffix(&mut result, expr);
    result
}

/// Generate Vue's production CSS variable name for a scoped SFC v-bind().
#[doc(hidden)]
pub fn prod_scoped_v_bind_name(id: &str, expr: &str) -> String {
    let hash = hash_sum_string_pair(id, expr);
    let mut result = String::with_capacity(8);
    write_hash_sum_hex(&mut result, hash);
    if result
        .as_bytes()
        .first()
        .is_some_and(|byte| byte.is_ascii_digit())
    {
        result.insert(0, 'v');
    }
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

fn hash_sum_string_pair(first: &str, second: &str) -> u64 {
    let hash = hash_sum_fold(0, "[object String]");
    let hash = hash_sum_fold(hash, "string");
    hash_sum_fold_pair(hash, first, second)
}

fn hash_sum_fold(input: u64, text: &str) -> u64 {
    if text.is_empty() {
        return input;
    }

    let mut hash = input as u32;
    for unit in text.encode_utf16() {
        hash = hash
            .wrapping_shl(5)
            .wrapping_sub(hash)
            .wrapping_add(u32::from(unit));
    }

    let signed = hash as i32;
    if signed < 0 {
        (-(i64::from(signed)) * 2) as u64
    } else {
        signed as u64
    }
}

fn hash_sum_fold_pair(input: u64, first: &str, second: &str) -> u64 {
    if first.is_empty() && second.is_empty() {
        return input;
    }

    let mut hash = input as u32;
    for unit in first.encode_utf16().chain(second.encode_utf16()) {
        hash = hash
            .wrapping_shl(5)
            .wrapping_sub(hash)
            .wrapping_add(u32::from(unit));
    }

    normalize_hash_sum_i32(hash)
}

fn write_hash_sum_hex(out: &mut String, value: u64) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut started = false;
    for shift in (0..64).step_by(4).rev() {
        let digit = ((value >> shift) & 0xF) as usize;
        if digit != 0 || started {
            out.push(HEX[digit] as char);
            started = true;
        }
    }
    if !started {
        out.push('0');
    }
    while out.len() < 8 {
        out.insert(0, '0');
    }
}

fn normalize_hash_sum_i32(hash: u32) -> u64 {
    let signed = hash as i32;
    if signed < 0 {
        (-(i64::from(signed)) * 2) as u64
    } else {
        signed as u64
    }
}

/// Write v-bind variable hash to output
fn write_v_bind_hash(out: &mut ArenaVec<u8>, expr: &str) {
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
fn write_hex_u32(out: &mut ArenaVec<u8>, val: u32) {
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
