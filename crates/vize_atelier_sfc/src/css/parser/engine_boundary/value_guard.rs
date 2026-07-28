//! Pre-parse guard for math-function shapes that crash LightningCSS.
//!
//! `Percentage::parse` hits `unreachable!()` when a math function in a
//! percentage-typed slot does not constant-fold to a plain percentage
//! (#3276, #3280; retirement tracked in #3295). The release profile builds
//! with `panic = "abort"`, so the `engine_boundary` catch cannot save shipped
//! binaries — the crashing shapes must be rejected before they reach the
//! engine.
//!
//! The exact panic surface is LightningCSS's calc type algebra, which a
//! byte scanner cannot reproduce. This guard covers the realistic authoring
//! surface instead: inside color functions and opacity-family declarations it
//! rejects the empirically crashing argument shapes, accepting two documented
//! trade-offs:
//!
//! - slot-blind over-rejection: `hsl(sign(-50%) 0% 0%)` parses today (the hue
//!   slot is not percentage-typed) but is rejected because the guard treats
//!   the whole color function as a percentage context;
//! - exotic under-coverage: percentage-typed slots outside the guarded
//!   contexts (`border-image-slice: abs(-50%)`) still reach the engine and
//!   rely on `engine_boundary` in unwinding profiles.

use vize_carton::String;

/// Error reported when a guarded shape is rejected before parsing.
pub(crate) const MATH_FUNCTION_GUARD_ERROR: &str = "CSS parse error: unsupported math function in a percentage context (upstream lightningcss defect; see vize issue #3295)";

/// Color functions whose component lists are treated as percentage contexts.
const COLOR_FUNCTIONS: [&str; 10] = [
    "rgb", "rgba", "hsl", "hsla", "hwb", "lab", "lch", "oklab", "oklch", "color",
];

/// Properties whose values are `<alpha-value>` (number or percentage).
const OPACITY_PROPERTIES: [&str; 6] = [
    "opacity",
    "fill-opacity",
    "stroke-opacity",
    "stop-opacity",
    "flood-opacity",
    "shape-image-threshold",
];

/// Math functions that reach `Percentage::parse` with a symbolic calc tree.
const MATH_FUNCTIONS: [&str; 9] = [
    "sign", "abs", "mod", "rem", "round", "min", "max", "clamp", "hypot",
];

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}

/// Scan one math-function argument list starting after its `(`; returns the
/// index just past the matching `)` (or EOF) and whether the argument shape is
/// one of the crashing ones: a percentage argument next to a unitless number
/// argument, or a percentage into the single-argument `sign`/`abs`. Both
/// parentheses are consumed here, so the caller's depth tracking never sees
/// them.
fn math_function_arguments_crash(bytes: &[u8], mut i: usize, single_arg: bool) -> (usize, bool) {
    let mut depth = 1usize;
    let (mut any_percent, mut any_bare_number) = (false, false);
    let (mut arg_digits, mut arg_alpha, mut arg_percent) = (false, false, false);
    while i < bytes.len() && depth > 0 {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    i += 1;
                    break;
                }
            }
            b',' if depth == 1 => {
                any_percent |= arg_percent;
                any_bare_number |= arg_digits && !arg_alpha && !arg_percent;
                (arg_digits, arg_alpha, arg_percent) = (false, false, false);
            }
            b'%' => arg_percent = true,
            b if b.is_ascii_digit() => arg_digits = true,
            b if b.is_ascii_alphabetic() => arg_alpha = true,
            _ => {}
        }
        i += 1;
    }
    any_percent |= arg_percent;
    any_bare_number |= arg_digits && !arg_alpha && !arg_percent;
    let crashes = any_percent && (single_arg || any_bare_number);
    (i.min(bytes.len()), crashes)
}

/// Returns true when `css` contains a math-function shape that would panic
/// inside LightningCSS in a color-function or opacity-family context.
pub(crate) fn css_contains_crashing_math_function(css: &str) -> bool {
    let lower: String = css.to_ascii_lowercase().into();
    let bytes = lower.as_bytes();
    let mut i = 0;
    // Depth of the innermost guarded percentage context, if any: a color
    // function's parentheses or an opacity-family declaration value.
    let mut context_depth: Option<usize> = None;
    let mut paren_depth = 0usize;
    let mut in_opacity_value = false;
    while i < bytes.len() {
        let b = bytes[i];
        match b {
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(bytes.len());
                continue;
            }
            b'"' | b'\'' => {
                i += 1;
                while i < bytes.len() && bytes[i] != b && bytes[i] != b'\n' {
                    i += if bytes[i] == b'\\' { 2 } else { 1 };
                }
                i = (i + 1).min(bytes.len());
                continue;
            }
            b'(' => paren_depth += 1,
            b')' => {
                paren_depth = paren_depth.saturating_sub(1);
                if context_depth.is_some_and(|depth| paren_depth < depth) {
                    context_depth = None;
                }
            }
            b';' | b'}' | b'{' => in_opacity_value = false,
            _ if is_ident_byte(b) => {
                let start = i;
                while i < bytes.len() && is_ident_byte(bytes[i]) {
                    i += 1;
                }
                let ident = &lower[start..i];
                let followed_by_paren = bytes.get(i) == Some(&b'(');
                if followed_by_paren {
                    if (context_depth.is_some() || in_opacity_value)
                        && MATH_FUNCTIONS.contains(&ident)
                    {
                        let single_arg = matches!(ident, "sign" | "abs");
                        let (_, crashes) = math_function_arguments_crash(bytes, i + 1, single_arg);
                        if crashes {
                            return true;
                        }
                        // Fall through without skipping the span so nested
                        // math functions (`min(abs(-50%), 60%)`) get their own
                        // argument check.
                    }
                    if context_depth.is_none() && COLOR_FUNCTIONS.contains(&ident) {
                        context_depth = Some(paren_depth + 1);
                    }
                } else if OPACITY_PROPERTIES.contains(&ident) && paren_depth == 0 {
                    let mut j = i;
                    while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                        j += 1;
                    }
                    if bytes.get(j) == Some(&b':') {
                        in_opacity_value = true;
                    }
                }
                continue;
            }
            _ => {}
        }
        i += 1;
    }
    false
}
