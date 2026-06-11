//! Vue 2 pipe-filter parsing (`{{ msg | capitalize }}`, `:id="raw | formatId"`).
//!
//! Legacy-only: filters were removed in Vue 3, where `|` is the bitwise-OR
//! operator. This module is compiled only behind the `legacy` cargo feature and
//! is consulted by [`crate::transforms::transform_expression::process_expression`]
//! solely when the resolved dialect advertises
//! [`supports_filters`](vize_armature::legacy::LegacyDialectCapabilities::supports_filters)
//! (Vue 2 / 2.7). For every other dialect — and for any build without the
//! `legacy` feature — this module is never reached and a `|`-containing
//! expression stays byte-identical to today's Vue 3 bitwise-OR output.
//!
//! The split + rewrite mirror `@vue/compiler-core`'s `parseFilter` /
//! `wrapFilter` (the `transformFilter` compat transform):
//!
//! - `a | f`      -> `_filter_f(a)`
//! - `a | f(b)`   -> `_filter_f(a,b)`
//! - `a | f | g`  -> `_filter_g(_filter_f(a))`
//!
//! Top-level `|` only: pipes inside strings, template literals, regex
//! literals, or `()[]{}` nesting are not filter separators, and `||` is the
//! logical-OR operator, never a filter.

use vize_carton::String;

use super::transform_expression::is_simple_identifier;

/// A parsed Vue 2 filter expression: the base expression and the ordered
/// filter chain applied to it (outermost last).
pub(crate) struct FilterExpression {
    /// The base expression text the filters are applied to (trimmed).
    pub(crate) base: String,
    /// Each filter as written after a top-level `|`, trimmed
    /// (e.g. `"capitalize"`, `"f(b)"`).
    pub(crate) filters: std::vec::Vec<String>,
}

/// Returns whether `c` can legally precede a `/` that starts a regex literal,
/// mirroring `@vue/compiler-core`'s `validDivisionCharRE = /[\w).+\-_$\]]/`:
/// a `/` after one of these is division, otherwise it opens a regex.
fn is_valid_division_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || matches!(c, ')' | '.' | '+' | '-' | '$' | ']')
}

/// Split a directive/interpolation expression into its base expression and
/// Vue 2 filter chain, mirroring `@vue/compiler-core`'s `parseFilter`.
///
/// Returns `None` when the expression contains no top-level filter pipe — the
/// caller then leaves the expression exactly as-is (so `a || b`, `a | b` under
/// a non-filter dialect, etc. are untouched). Returns `Some` only when at least
/// one real filter was found.
pub(crate) fn parse_filters(exp: &str) -> Option<FilterExpression> {
    let bytes = exp.as_bytes();
    let len = bytes.len();

    let mut in_single = false;
    let mut in_double = false;
    let mut in_template = false;
    let mut in_regex = false;
    let mut curly: u32 = 0;
    let mut square: u32 = 0;
    let mut paren: u32 = 0;

    let mut last_filter_index = 0usize;
    // `None` until the first top-level pipe splits off the base expression.
    let mut expression: Option<String> = None;
    let mut filters: std::vec::Vec<String> = std::vec::Vec::new();
    let mut prev: u8 = 0;

    let mut i = 0usize;
    while i < len {
        let c = bytes[i];

        if in_single {
            if c == b'\'' && prev != b'\\' {
                in_single = false;
            }
        } else if in_double {
            if c == b'"' && prev != b'\\' {
                in_double = false;
            }
        } else if in_template {
            if c == b'`' && prev != b'\\' {
                in_template = false;
            }
        } else if in_regex {
            if c == b'/' && prev != b'\\' {
                in_regex = false;
            }
        } else if c == b'|'
            && bytes.get(i + 1).copied() != Some(b'|')
            && (i == 0 || bytes[i - 1] != b'|')
            && curly == 0
            && square == 0
            && paren == 0
        {
            // Top-level filter pipe.
            if expression.is_none() {
                last_filter_index = i + 1;
                expression = Some(String::new(exp[..i].trim()));
            } else {
                filters.push(String::new(exp[last_filter_index..i].trim()));
                last_filter_index = i + 1;
            }
        } else {
            match c {
                b'"' => in_double = true,
                b'\'' => in_single = true,
                b'`' => in_template = true,
                b'(' => paren += 1,
                b')' => paren = paren.saturating_sub(1),
                b'[' => square += 1,
                b']' => square = square.saturating_sub(1),
                b'{' => curly += 1,
                b'}' => curly = curly.saturating_sub(1),
                _ => {}
            }
            if c == b'/' {
                // Look back past spaces for the previous non-space char to
                // decide division vs. regex (mirrors `validDivisionCharRE`).
                let mut j = i as isize - 1;
                let mut p: Option<char> = None;
                while j >= 0 {
                    let pc = exp[j as usize..].chars().next().unwrap_or(' ');
                    if pc != ' ' {
                        p = Some(pc);
                        break;
                    }
                    j -= 1;
                }
                if p.is_none_or(|pc| !is_valid_division_char(pc)) {
                    in_regex = true;
                }
            }
        }

        prev = c;
        i += 1;
    }

    match &mut expression {
        None => return None,
        Some(_) if last_filter_index != 0 => {
            filters.push(String::new(exp[last_filter_index..].trim()));
        }
        Some(_) => {}
    }

    let base = expression?;
    if filters.is_empty() {
        return None;
    }
    Some(FilterExpression { base, filters })
}

/// Wrap `exp` with one filter, mirroring `@vue/compiler-core`'s `wrapFilter`.
///
/// - `f`     -> `_filter_f(exp)`
/// - `f(b)`  -> `_filter_f(exp,b)`
/// - `f()`   -> `_filter_f(exp)`
///
/// `filter_id` is the already-resolved asset identifier
/// (`_filter_<validated-name>`). Returns the wrapped expression and the raw
/// filter name for asset registration. Returns `None` for a filter whose name
/// is not a valid identifier (e.g. empty), in which case the caller bails out
/// and leaves the expression untouched.
pub(crate) fn wrap_filter(exp: &str, filter: &str, filter_id: &str) -> Option<String> {
    match filter.find('(') {
        None => {
            // Bare filter name: `_filter_f(exp)`.
            let mut out = String::with_capacity(filter_id.len() + exp.len() + 2);
            out.push_str(filter_id);
            out.push('(');
            out.push_str(exp);
            out.push(')');
            Some(out)
        }
        Some(idx) => {
            // Call-style filter `f(args)`: `_filter_f(exp,args` with the
            // trailing `)` carried over from `args` (Vue 2 call syntax).
            let args = &filter[idx + 1..];
            let mut out = String::with_capacity(filter_id.len() + exp.len() + args.len() + 3);
            out.push_str(filter_id);
            out.push('(');
            out.push_str(exp);
            if args == ")" {
                // `f()` -> `_filter_f(exp)`
                out.push(')');
            } else {
                // `f(b, c)` -> `_filter_f(exp,b, c)` (args still holds the `)`)
                out.push(',');
                out.push_str(args);
            }
            Some(out)
        }
    }
}

/// Extract the bare filter name from a (possibly call-style) filter segment,
/// e.g. `"f(b)"` -> `"f"`, `"capitalize"` -> `"capitalize"`. Returns `None`
/// for an empty/invalid name so the caller can bail out unchanged.
pub(crate) fn filter_name(filter: &str) -> Option<&str> {
    let name = match filter.find('(') {
        Some(idx) => &filter[..idx],
        None => filter,
    };
    let name = name.trim();
    // Vue resolves whatever name appears; require a non-empty, identifier-like
    // token so we never emit `_filter_(` for malformed input. `-` is allowed
    // because `toValidAssetId` maps it (foo-bar -> _filter_foo_bar).
    if name.is_empty() {
        return None;
    }
    let dashless = name.replace('-', "_");
    if is_simple_identifier(&dashless) {
        Some(name)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Assert a filter split: `exp` must parse to `expected_base` plus the
    /// `expected_filters` chain. Comparisons stay on `&str` to keep the test
    /// free of the crate's disallowed std `String`.
    fn assert_split(exp: &str, expected_base: &str, expected_filters: &[&str]) {
        let parsed = parse_filters(exp).expect("expected a filter split");
        assert_eq!(parsed.base.as_str(), expected_base);
        assert_eq!(parsed.filters.len(), expected_filters.len());
        for (got, want) in parsed.filters.iter().zip(expected_filters) {
            assert_eq!(got.as_str(), *want);
        }
    }

    #[test]
    fn no_pipe_is_none() {
        assert!(parse_filters("message").is_none());
        assert!(parse_filters("a + b").is_none());
    }

    #[test]
    fn logical_or_is_not_a_filter() {
        assert!(parse_filters("a || b").is_none());
        assert!(parse_filters("a || b || c").is_none());
    }

    #[test]
    fn bitwise_or_outside_filter_dialect_is_caller_gated() {
        // `a | b` *is* a filter split here; the dialect gate in
        // process_expression is what keeps Vue 3 bitwise-or untouched.
        assert_split("a | b", "a", &["b"]);
    }

    #[test]
    fn single_filter() {
        assert_split("message | capitalize", "message", &["capitalize"]);
    }

    #[test]
    fn filter_with_args() {
        assert_split("a | f(b)", "a", &["f(b)"]);
    }

    #[test]
    fn filter_chain() {
        assert_split("a | f | g(c)", "a", &["f", "g(c)"]);
    }

    #[test]
    fn pipe_inside_string_is_ignored() {
        assert_split("a | f('a|b')", "a", &["f('a|b')"]);
        // A pipe living only inside a string must not split anything.
        assert!(parse_filters("'x | y'").is_none());
        assert!(parse_filters("`x | y`").is_none());
    }

    #[test]
    fn pipe_inside_brackets_is_ignored() {
        assert!(parse_filters("[a | b]").is_none());
        assert!(parse_filters("f(a | b)").is_none());
        assert!(parse_filters("{ a: b | c }").is_none());
    }

    #[test]
    fn base_can_be_a_call() {
        assert_split("f(a) | g", "f(a)", &["g"]);
    }

    #[test]
    fn regex_literal_pipe_is_ignored() {
        // `/a|b/` is a regex literal; its `|` is not a filter pipe.
        assert_split("x.match(/a|b/) | g", "x.match(/a|b/)", &["g"]);
    }

    #[test]
    fn wrap_filter_bare() {
        assert_eq!(
            wrap_filter("message", "capitalize", "_filter_capitalize").unwrap(),
            "_filter_capitalize(message)"
        );
    }

    #[test]
    fn wrap_filter_call() {
        assert_eq!(
            wrap_filter("a", "f(b)", "_filter_f").unwrap(),
            "_filter_f(a,b)"
        );
    }

    #[test]
    fn wrap_filter_empty_call() {
        assert_eq!(
            wrap_filter("a", "f()", "_filter_f").unwrap(),
            "_filter_f(a)"
        );
    }

    #[test]
    fn wrap_filter_multi_args_preserves_spacing() {
        assert_eq!(
            wrap_filter("a", "f(b, c)", "_filter_f").unwrap(),
            "_filter_f(a,b, c)"
        );
    }

    #[test]
    fn filter_name_extraction() {
        assert_eq!(filter_name("capitalize"), Some("capitalize"));
        assert_eq!(filter_name("f(b)"), Some("f"));
        assert_eq!(filter_name("foo-bar"), Some("foo-bar"));
        assert_eq!(filter_name("(x)"), None);
        assert_eq!(filter_name(""), None);
    }
}
