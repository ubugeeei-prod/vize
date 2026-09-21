//! Constant JavaScript values in pug attribute and code positions.
//!
//! pug folds constant expressions at compile time (`constantinople`) and
//! runs everything else. The Vue lowering only accepts what folds to a
//! plain value without a JavaScript engine — string literals (`"…"`,
//! `'…'`, a substitution-free `` `…` ``), `true`, `false`, `null`,
//! `undefined` and plain integers — cooked with ECMAScript's escape rules.
//! Anything else is executable pug and refused by the caller.

use vize_s0::String;

/// A folded constant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Lit {
    Str(String),
    Bool(bool),
    Null,
    Undefined,
    /// A plain non-negative integer, already in JavaScript's
    /// `Number#toString` spelling.
    Int(String),
}

impl Lit {
    /// `x || ''`-style truthiness.
    pub(crate) fn truthy(&self) -> bool {
        match self {
            Lit::Str(text) => !text.is_empty(),
            Lit::Bool(value) => *value,
            Lit::Null | Lit::Undefined => false,
            Lit::Int(digits) => digits != "0",
        }
    }

    /// `'' + value`.
    pub(crate) fn to_js_string(&self) -> String {
        match self {
            Lit::Str(text) | Lit::Int(text) => text.clone(),
            Lit::Bool(true) => String::from("true"),
            Lit::Bool(false) => String::from("false"),
            Lit::Null => String::from("null"),
            Lit::Undefined => String::from("undefined"),
        }
    }
}

/// Fold `source` (an attribute value or `=` code, as authored) or `None`.
pub(crate) fn evaluate(source: &str) -> Option<Lit> {
    let text = source.trim_matches(is_js_space);
    match text {
        "true" => return Some(Lit::Bool(true)),
        "false" => return Some(Lit::Bool(false)),
        "null" => return Some(Lit::Null),
        "undefined" => return Some(Lit::Undefined),
        _ => {}
    }
    let first = text.chars().next()?;
    if matches!(first, '"' | '\'' | '`') {
        return string_literal(text, first).map(Lit::Str);
    }
    let digits = text.bytes().all(|b| b.is_ascii_digit());
    let plain = text.len() <= 15 && (text == "0" || !text.starts_with('0'));
    (digits && plain).then(|| Lit::Int(String::from(text)))
}

pub(super) fn is_js_space(ch: char) -> bool {
    matches!(
        ch,
        '\t' | '\n' | '\u{b}' | '\u{c}' | '\r' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200a}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202f}'
                | '\u{205f}'
                | '\u{3000}'
                | '\u{feff}'
    )
}

/// The cooked value of a literal that spans all of `text`. Raw line
/// breaks are refused (a string cannot hold one; a template literal can,
/// but its bytes would depend on the SFC dedent the S1 view applies).
fn string_literal(text: &str, quote: char) -> Option<String> {
    let body = text.get(1..)?.strip_suffix(quote)?;
    let template = quote == '`';
    let mut out = String::with_capacity(body.len());
    let mut chars = body.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\n' | '\r' => return None,
            c if c == quote => return None,
            '$' if template && chars.peek() == Some(&'{') => return None,
            '\\' => {
                let escaped = chars.next()?;
                match escaped {
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    'b' => out.push('\u{8}'),
                    'f' => out.push('\u{c}'),
                    'v' => out.push('\u{b}'),
                    '0' if !chars.peek().is_some_and(char::is_ascii_digit) => out.push('\0'),
                    '0'..='7' => return None,
                    '8' | '9' if template => return None,
                    'x' => out.push(hex_escape(&mut chars, 2)?),
                    'u' => out.push(unicode_escape(&mut chars)?),
                    '\n' | '\r' | '\u{2028}' | '\u{2029}' => return None,
                    other => out.push(other),
                }
            }
            other => out.push(other),
        }
    }
    Some(out)
}

fn hex_digits(chars: &mut core::iter::Peekable<core::str::Chars<'_>>, count: usize) -> Option<u32> {
    let mut value = 0u32;
    for _ in 0..count {
        value = value * 16 + chars.next()?.to_digit(16)?;
    }
    Some(value)
}

fn hex_escape(
    chars: &mut core::iter::Peekable<core::str::Chars<'_>>,
    count: usize,
) -> Option<char> {
    char::from_u32(hex_digits(chars, count)?)
}

/// `\uXXXX` (pairing a surrogate escape with a following low one) or
/// `\u{X…}`; a lone surrogate has no `String` form and is refused.
fn unicode_escape(chars: &mut core::iter::Peekable<core::str::Chars<'_>>) -> Option<char> {
    let unit = if chars.peek() == Some(&'{') {
        chars.next();
        let mut value = 0u32;
        let mut digits = 0;
        loop {
            let ch = chars.next()?;
            if ch == '}' {
                break;
            }
            value = value.checked_mul(16)?.checked_add(ch.to_digit(16)?)?;
            digits += 1;
        }
        if digits == 0 || value > 0x10_ffff {
            return None;
        }
        value
    } else {
        hex_digits(chars, 4)?
    };
    if (0xd800..0xdc00).contains(&unit) {
        let mut rest = chars.clone();
        if rest.next()? != '\\' || rest.next()? != 'u' {
            return None;
        }
        let low = hex_digits(&mut rest, 4)?;
        if !(0xdc00..0xe000).contains(&low) {
            return None;
        }
        *chars = rest;
        return char::from_u32(0x10000 + ((unit - 0xd800) << 10) + (low - 0xdc00));
    }
    char::from_u32(unit)
}

#[cfg(test)]
mod tests {
    use super::{Lit, evaluate};
    use vize_s0::String;

    fn text(value: &str) -> Option<Lit> {
        Some(Lit::Str(String::from(value)))
    }

    #[test]
    fn folds_the_accepted_constants() {
        assert_eq!(evaluate(" \"a < b\" "), text("a < b"));
        assert_eq!(evaluate("'x\"y'"), text("x\"y"));
        assert_eq!(evaluate("`t`"), text("t"));
        assert_eq!(
            evaluate("'\\d\\n\\x41\\u0042\\u{43}\\uD83D\\uDE00'"),
            text("d\nABC😀")
        );
        assert_eq!(evaluate("true"), Some(Lit::Bool(true)));
        assert_eq!(evaluate("null"), Some(Lit::Null));
        assert_eq!(evaluate("12"), Some(Lit::Int(String::from("12"))));
    }

    #[test]
    fn refuses_what_needs_an_engine() {
        for source in [
            "a",
            "\"a\" + \"b\"",
            "`${x}`",
            "'\\1'",
            "012",
            "'a\nb'",
            "'unterminated",
            "'a'b'",
            "1.5",
            "-1",
            "'\\uD800'",
        ] {
            assert_eq!(evaluate(source), None, "{source}");
        }
    }
}
