#![no_main]

// CSS parser fuzz target.
//
// Exercises the vize_atelier_sfc Lightning CSS integration through the public
// serialized-AST API. Syntax errors should be returned in CssAstResult; panics
// are always a bug in the integration layer or upstream parser boundary.
//
// Three upstream defect classes are skip-listed below until their fixes land
// (#3276, #3280, #3926; retirement tracked in #3295). Production is protected
// by the `catch_unwind` boundary in
// `vize_atelier_sfc::css::parser::engine_boundary`; the skips exist because
// libfuzzer-sys aborts inside its panic hook (a caught-and-reported upstream
// panic still registers as a fuzz crash) and counts a slow parse against its
// timeout budget. Every skip is deliberately broad — it costs fuzz coverage,
// never product behavior.
use libfuzzer_sys::fuzz_target;
use vize_atelier_sfc::{CssCompileOptions, parse_css_ast};

/// Math functions that reach `Percentage::parse` with a symbolic calc tree
/// when a `%` is involved, hitting upstream `unreachable!()` (#3276). `calc()`
/// itself resolves; the panics need one of these names, so nested shapes like
/// `calc(sign(-50%))` are still matched via the inner function.
const PERCENTAGE_MATH_FUNCTIONS: [&str; 9] = [
    "sign(", "abs(", "mod(", "rem(", "round(", "min(", "max(", "clamp(", "hypot(",
];

/// Returns true when `name` occurs and a `%` appears before its parenthesis
/// group closes (unterminated groups run to end of input, matching the
/// recovering parser).
fn function_arguments_contain_percent(lower: &str, name: &str) -> bool {
    let bytes = lower.as_bytes();
    let mut from = 0;
    while let Some(found) = lower[from..].find(name) {
        let open = from + found + name.len() - 1;
        let mut depth = 1usize;
        let mut i = open + 1;
        while i < bytes.len() && depth > 0 {
            match bytes[i] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                b'%' => return true,
                _ => {}
            }
            i += 1;
        }
        from = open + 1;
    }
    false
}

/// Returns true for numeric tokens big enough to go non-finite in the f32
/// value pipeline (55-digit literals, `1e40`, `calc(1e20 * 1e20)`), which
/// trips the upstream hue `debug_assert` under fuzzing profiles (#3280).
/// Digit runs above 20 or exponents with two or more digits are skipped;
/// products of single-digit exponents can still overflow, so a future
/// reproducer of that shape widens this filter rather than refuting it.
fn has_oversized_numeric_token(lower: &str) -> bool {
    let bytes = lower.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
                i += 1;
            }
            if i - start > 20 {
                return true;
            }
            if i < bytes.len() && bytes[i] == b'e' {
                let mut j = i + 1;
                if j < bytes.len() && (bytes[j] == b'+' || bytes[j] == b'-') {
                    j += 1;
                }
                let exponent_start = j;
                while j < bytes.len() && bytes[j].is_ascii_digit() {
                    j += 1;
                }
                if j - exponent_start >= 2 {
                    return true;
                }
                i = j;
            }
        } else {
            i += 1;
        }
    }
    false
}

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

/// Returns true when `name` occurs and its parenthesis group is still open at
/// end of input. Parens inside strings and comments count like the tokenizer's
/// wouldn't — the same deliberate coarseness as the other skip scanners.
fn function_group_unterminated(lower: &str, name: &str) -> bool {
    let bytes = lower.as_bytes();
    let mut from = 0;
    while let Some(found) = lower[from..].find(name) {
        let open = from + found + name.len() - 1;
        let mut depth = 1usize;
        let mut i = open + 1;
        while i < bytes.len() && depth > 0 {
            match bytes[i] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                _ => {}
            }
            i += 1;
        }
        if depth > 0 {
            return true;
        }
        from = open + 1;
    }
    false
}

/// Known upstream defect shapes (#3276, #3280 panics; #3926 timeout): skip
/// them so fuzzing keeps hunting new engine bugs without re-reporting the
/// documented defects.
///
/// The #3926 class is pathological backtracking, not a hang: unterminated
/// color functions with rule-block braces interleaved into the open groups
/// made a 2.6 KB input parse in ~18 s (debug) / ~2.3 s (release), which the
/// instrumented fuzz build converts into a timeout finding. The skip keys on
/// the simpler superset — any unterminated color-function group — because
/// valid stylesheets always close them; a future reproducer that backtracks
/// without a color function widens this filter rather than refuting it.
fn hits_known_upstream_defect_shape(source: &str) -> bool {
    let lower = source.to_ascii_lowercase();
    if lower.contains('%')
        && PERCENTAGE_MATH_FUNCTIONS
            .iter()
            .any(|name| function_arguments_contain_percent(&lower, name))
    {
        return true;
    }
    if COLOR_FUNCTIONS
        .iter()
        .any(|name| function_group_unterminated(&lower, name))
    {
        return true;
    }
    has_oversized_numeric_token(&lower)
}

fuzz_target!(|data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };
    if hits_known_upstream_defect_shape(source) {
        return;
    }

    let _ = parse_css_ast(source, &CssCompileOptions::default());
});
