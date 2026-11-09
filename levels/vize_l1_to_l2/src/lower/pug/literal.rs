//! JavaScript string literals in pug code, cooked with ECMAScript's
//! escape rules, and the whitespace set pug's generator trims.

use vize_l0::String;

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
/// but its bytes would depend on the SFC dedent the L1 view applies).
pub(super) fn string_literal(text: &str, quote: char) -> Option<String> {
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
    use super::string_literal;

    fn cook(source: &str) -> Option<vize_l0::String> {
        string_literal(source, source.chars().next()?)
    }

    #[test]
    fn cooks_escapes_like_javascript() {
        assert_eq!(cook("\"a < b\"").as_deref(), Some("a < b"));
        assert_eq!(cook("'x\"y'").as_deref(), Some("x\"y"));
        assert_eq!(cook("`t`").as_deref(), Some("t"));
        assert_eq!(
            cook("'\\d\\n\\x41\\u0042\\u{43}\\uD83D\\uDE00'").as_deref(),
            Some("d\nABC😀")
        );
    }

    #[test]
    fn refuses_what_has_no_string_value() {
        for source in [
            "`${x}`",
            "'\\1'",
            "'a\nb'",
            "'unterminated",
            "'a'b'",
            "'\\uD800'",
        ] {
            assert_eq!(cook(source), None, "{source}");
        }
    }
}
