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
                result.extend_from_slice(css_bytes.get(pos..).unwrap_or_default());
                break;
            };

            if let Some(end) = find_matching_paren(after_open) {
                // Copy everything before v-bind(
                result.extend_from_slice(css_bytes.get(pos..actual_pos).unwrap_or_default());

                // Extract expression
                let Some(expr_str) = after_open.get(..end).map(str::trim) else {
                    pos = start + end + 1;
                    result.extend_from_slice(css_bytes.get(actual_pos..pos).unwrap_or_default());
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
                result.extend_from_slice(css_bytes.get(pos..).unwrap_or_default());
                break;
            }
        } else {
            result.extend_from_slice(css_bytes.get(pos..).unwrap_or_default());
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

/// Authored JavaScript expression ranges, excluding CSS whitespace and quotes.
/// Shares the compiler scanner, including its comment/string and nesting rules.
#[doc(hidden)]
pub fn v_bind_expression_ranges(css: &str) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    let mut pos = 0;
    while let Some(open) = find_next_v_bind(css, pos) {
        let start = open + 7;
        let Some(end) = find_matching_paren(css.get(start..).unwrap_or_default()) else {
            break;
        };
        let raw = css.get(start..start + end).unwrap_or_default();
        let trimmed = raw.trim();
        let expression = trim_outer_quotes(trimmed);
        let quote = usize::from(expression.len() != trimmed.len());
        let start = start + raw.len() - raw.trim_start().len() + quote;
        ranges.push(start..start + expression.len());
        pos = open + 7 + end + 1;
    }
    ranges
}

pub(crate) fn trim_outer_quotes(expr: &str) -> &str {
    let bytes = expr.as_bytes();
    if bytes.len() >= 2
        && matches!(bytes.first(), Some(b'"' | b'\''))
        && bytes.first() == bytes.last()
    {
        expr.get(1..expr.len() - 1).unwrap_or_default()
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
    let mut started = false;
    for shift in (0..64).step_by(4).rev() {
        let digit = ((value >> shift) & 0xF) as u8;
        if digit != 0 || started {
            out.push(hex_digit(digit) as char);
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
    for shift in [28, 24, 20, 16, 12, 8, 4, 0] {
        out.push(hex_digit(((val >> shift) & 0xF) as u8));
    }
}

/// Lowercase hex digit for a nibble (`0..16`).
#[inline]
const fn hex_digit(nibble: u8) -> u8 {
    if nibble < 10 {
        b'0' + nibble
    } else {
        b'a' + (nibble - 10)
    }
}
