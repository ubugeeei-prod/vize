//! `rgb()` / `rgba()` recognition and component parsing.

use super::{ColorLiteral, lex::decode_identifier};

pub(super) fn is_function_name(content: &str, start: usize, end: usize) -> bool {
    let mut decoded = [0u8; 4];
    let Some(len) = decode_identifier(content.as_bytes(), start, end, &mut decoded) else {
        return false;
    };
    decoded[..len].eq_ignore_ascii_case(b"rgb") || decoded[..len].eq_ignore_ascii_case(b"rgba")
}

/// Comma- or space-separated notation with an optional alpha after `,` or `/`.
pub(super) fn literal(
    content: &str,
    start: usize,
    identifier_end: usize,
    limit: usize,
) -> Option<ColorLiteral> {
    #[cfg(test)]
    super::RGB_PROBES.set(super::RGB_PROBES.get() + 1);
    let arguments_start = identifier_end + 1;
    let close = content[arguments_start..limit].find(')')? + arguments_start;
    let end = close + 1;

    let mut components = content[arguments_start..close]
        .split(|ch: char| ch == ',' || ch == '/' || ch.is_ascii_whitespace())
        .filter(|part| !part.is_empty());
    let red = channel(components.next()?, 255.0)?;
    let green = channel(components.next()?, 255.0)?;
    let blue = channel(components.next()?, 255.0)?;
    let alpha = components
        .next()
        .map_or(Some(1.0), |part| channel(part, 1.0))?;
    if components.next().is_some() {
        return None;
    }

    Some(ColorLiteral {
        start,
        end,
        red,
        green,
        blue,
        alpha,
    })
}

/// One component normalised to `0.0..=1.0`; `full` maps a number to one.
pub(super) fn channel(part: &str, full: f32) -> Option<f32> {
    let part = trim_css_whitespace(part);
    let value = match part.strip_suffix('%') {
        Some(percent) => parse_css_number(percent)? / 100.0,
        None => parse_css_number(part)? / full,
    };
    value.is_finite().then(|| value.clamp(0.0, 1.0))
}

pub(super) fn trim_css_whitespace(part: &str) -> &str {
    part.trim_matches(|character| matches!(character, ' ' | '\t' | '\n' | '\r' | '\u{000c}'))
}

/// CSS Syntax's `<number>` spelling, excluding delimiters Rust also accepts
/// (for example the trailing dot in `1.`).
pub(super) fn parse_css_number(part: &str) -> Option<f32> {
    let bytes = part.as_bytes();
    let mut cursor = usize::from(matches!(bytes.first(), Some(b'+') | Some(b'-')));
    let integer_start = cursor;
    while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
        cursor += 1;
    }
    let integer_digits = cursor - integer_start;

    let mut fraction_digits = 0;
    if bytes.get(cursor) == Some(&b'.') && bytes.get(cursor + 1).is_some_and(u8::is_ascii_digit) {
        cursor += 1;
        let fraction_start = cursor;
        while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
            cursor += 1;
        }
        fraction_digits = cursor - fraction_start;
    }
    if integer_digits + fraction_digits == 0 {
        return None;
    }

    if matches!(bytes.get(cursor), Some(b'e') | Some(b'E')) {
        cursor += 1;
        if matches!(bytes.get(cursor), Some(b'+') | Some(b'-')) {
            cursor += 1;
        }
        let exponent_start = cursor;
        while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
            cursor += 1;
        }
        if cursor == exponent_start {
            return None;
        }
    }
    if cursor != bytes.len() {
        return None;
    }
    part.parse::<f32>().ok().filter(|value| value.is_finite())
}

#[cfg(test)]
mod tests {
    use super::parse_css_number;

    #[test]
    fn css_number_spelling_requires_digits_after_dot_and_exponent() {
        for accepted in ["0", "+1", "-.5", "1.25", "1e2", "1E-2"] {
            assert!(parse_css_number(accepted).is_some(), "{accepted}");
        }
        for rejected in ["", ".", "1.", "1.e2", "1e", "1e+", " 1", "1 "] {
            assert_eq!(parse_css_number(rejected), None, "{rejected}");
        }
    }
}
