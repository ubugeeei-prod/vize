use super::json_error;
use crate::error::FormatError;
use std::iter::Peekable;
use std::str::Chars;
use vize_s0::String;

/// Scan a JSON number and copy it verbatim.
///
/// JSON numbers are `-? (0 | [1-9][0-9]*) (. [0-9]+)? ([eE] [+-]? [0-9]+)?`.
/// The formatter preserves the numeric token text, but still validates the
/// grammar so invalid JSON cannot be silently normalized into output.
pub(super) fn parse(iter: &mut Peekable<Chars<'_>>) -> Result<String, FormatError> {
    let mut out = String::default();

    if iter.peek().copied() == Some('-') {
        out.push('-');
        iter.next();
    }

    match iter.peek().copied() {
        Some('0') => {
            out.push('0');
            iter.next();
            if iter.peek().is_some_and(|c| c.is_ascii_digit()) {
                return Err(json_error("leading zero in number"));
            }
        }
        Some('1'..='9') => consume_digits(iter, &mut out),
        _ => return Err(json_error("expected digit in number")),
    }

    if iter.peek().copied() == Some('.') {
        out.push('.');
        iter.next();
        if !iter.peek().is_some_and(|c| c.is_ascii_digit()) {
            return Err(json_error("expected digit after decimal point"));
        }
        consume_digits(iter, &mut out);
    }

    if matches!(iter.peek().copied(), Some('e' | 'E')) {
        if let Some(exponent) = iter.next() {
            out.push(exponent);
        }
        if let Some(sign @ ('+' | '-')) = iter.peek().copied() {
            out.push(sign);
            iter.next();
        }
        if !iter.peek().is_some_and(|c| c.is_ascii_digit()) {
            return Err(json_error("expected digit in exponent"));
        }
        consume_digits(iter, &mut out);
    }

    Ok(out)
}

fn consume_digits(iter: &mut Peekable<Chars<'_>>, out: &mut String) {
    while let Some(c @ '0'..='9') = iter.peek().copied() {
        out.push(c);
        iter.next();
    }
}
