#![no_main]

// Fuzz the public recovery contract used by native release artifacts. Engine
// panics converted to explicit diagnostics are valid results; panics escaping
// the API still abort and produce a reproducer. Do not skip numeric/math inputs.
use libfuzzer_sys::fuzz_target;
use vize_atelier_sfc::{CssCompileOptions, parse_css_ast};

#[path = "../../../crates/vize_atelier_sfc/tests/support/css_fuzz_boundary.rs"]
mod boundary;

/// Color-entry functions recognized by the unparsed-value fallback. When one
/// of these opens a parenthesis group that never closes, the recovering parser
/// treats the entire remaining input as its arguments, and lightningcss's
/// `TokenList` → `UnresolvedColor` path re-attempts a color parse per
/// candidate token with nested `try_parse` rescans of the whole suffix.
const COLOR_FUNCTIONS: [&str; 12] = [
    "rgb(",
    "rgba(",
    "hsl(",
    "hsla(",
    "hwb(",
    "lab(",
    "lch(",
    "oklab(",
    "oklch(",
    "color(",
    "color-mix(",
    "light-dark(",
];

/// Returns the index just past the `/* ... */` comment starting at `i`, or
/// `bytes.len()` when it never closes.
fn skip_comment(bytes: &[u8], i: usize) -> usize {
    let mut j = i + 2;
    while j < bytes.len() {
        if bytes[j] == b'*' && bytes.get(j + 1) == Some(&b'/') {
            return j + 2;
        }
        j += 1;
    }
    bytes.len()
}

/// Returns the index just past the quoted string starting at `i` (whose byte is
/// the opening quote), or `bytes.len()` when it never closes.
fn skip_string(bytes: &[u8], i: usize) -> usize {
    let quote = bytes[i];
    let mut j = i + 1;
    while j < bytes.len() {
        match bytes[j] {
            b'\\' => j += 2,
            b if b == quote => return j + 1,
            _ => j += 1,
        }
    }
    bytes.len()
}

/// Returns true when `name` occurs and its parenthesis group is still open at
/// end of input. Parens inside CSS comments and quoted strings are skipped like
/// the tokenizer skips them, so `rgb(/* ) */` and `rgb(")")` still count as
/// unterminated; this mirrors the lexical handling of the production guard in
/// `vize_atelier_sfc::css::parser::engine_boundary::value_guard`.
fn function_group_unterminated(lower: &str, name: &str) -> bool {
    let bytes = lower.as_bytes();
    let mut from = 0;
    while let Some(found) = lower[from..].find(name) {
        let open = from + found + name.len() - 1;
        let mut depth = 1usize;
        let mut i = open + 1;
        while i < bytes.len() && depth > 0 {
            match bytes[i] {
                b'/' if bytes.get(i + 1) == Some(&b'*') => i = skip_comment(bytes, i),
                b'"' | b'\'' => i = skip_string(bytes, i),
                b'(' => {
                    depth += 1;
                    i += 1;
                }
                b')' => {
                    depth -= 1;
                    i += 1;
                }
                _ => i += 1,
            }
        }
        if depth > 0 {
            return true;
        }
        from = open + 1;
    }
    false
}

// Retain only the separately tracked upstream backtracking exclusion (#3926).
fn hits_known_timeout(source: &str) -> bool {
    let lower = source.to_ascii_lowercase();
    COLOR_FUNCTIONS
        .iter()
        .any(|name| function_group_unterminated(&lower, name))
}

fuzz_target!(init: boundary::install_hook(), |data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else { return; };
    if hits_known_timeout(source) { return; }
    if boundary::call(|| parse_css_ast(source, &CssCompileOptions::default())).is_err() {
        std::process::abort();
    }
});
