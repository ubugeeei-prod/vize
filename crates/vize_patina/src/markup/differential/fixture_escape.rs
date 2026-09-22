//! Cooked-string escapes for the rule-fixture scanner.
//!
//! `\xNN` is a 7-bit escape and `\u{...}` is a Unicode scalar, including
//! `_` separators. Byte and C strings are not `&str` templates.

use vize_s0::String;

use super::fixture_cursor::Cursor;

pub(super) fn string_prefix_len(source: &str) -> Option<usize> {
    for prefix in ["br", "cr", "b", "c", "r"] {
        if let Some(rest) = source.strip_prefix(prefix)
            && opens_string(rest)
        {
            return Some(prefix.len());
        }
    }
    source.starts_with('"').then_some(0)
}

fn opens_string(source: &str) -> bool {
    let hashes = hash_prefix(source);
    hashes < 256 && source[hashes..].starts_with('"')
}

pub(super) fn hash_prefix(source: &str) -> usize {
    source.bytes().take_while(|byte| *byte == b'#').count()
}

/// `\x` is a 7-bit escape: an octal digit, then a hex digit.
pub(super) fn decode_ascii_escape(cur: &mut Cursor<'_>) -> Option<char> {
    let mut chars = cur.rest().chars();
    let hi = chars.next()?;
    let lo = chars.next()?;
    if !matches!(hi, '0'..='7') || lo.to_digit(16).is_none() {
        return None;
    }
    let value = (hi.to_digit(16)? << 4) | lo.to_digit(16)?;
    cur.bump();
    cur.bump();
    char::from_u32(value)
}

/// `\u{...}` with up to six hex digits and optional `_` separators.
pub(super) fn decode_unicode_escape(cur: &mut Cursor<'_>) -> Option<char> {
    if cur.peek() != '{' {
        return None;
    }
    cur.bump();
    let start = cur.i;
    while !cur.eof() && cur.peek() != '}' {
        let ch = cur.peek();
        if ch != '_' && ch.to_digit(16).is_none() {
            return None;
        }
        cur.bump();
    }
    if cur.peek() != '}' {
        return None;
    }
    let body = &cur.source[start..cur.i];
    cur.bump();
    if !valid_unicode_body(body) {
        return None;
    }
    let mut value = 0u32;
    for ch in body.chars() {
        if let Some(digit) = ch.to_digit(16) {
            value = value.checked_mul(16)?.checked_add(digit)?;
        }
    }
    char::from_u32(value)
}

fn valid_unicode_body(body: &str) -> bool {
    let mut groups = 0;
    let mut chars = body.chars().peekable();
    while chars.peek().is_some() {
        if chars.next().is_none_or(|ch| ch.to_digit(16).is_none()) {
            return false;
        }
        groups += 1;
        while chars.peek().is_some_and(|ch| *ch == '_') {
            chars.next();
        }
    }
    (1..=6).contains(&groups)
}

pub(super) fn skip_continuation_ws(cur: &mut Cursor<'_>) {
    while !cur.eof() && matches!(cur.peek(), '\t' | '\n' | '\r' | ' ') {
        cur.bump();
    }
}

pub(super) fn normalize_newlines(text: &str) -> String {
    let mut out = String::default();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\r' && chars.peek() == Some(&'\n') {
            chars.next();
            out.push('\n');
        } else {
            out.push(ch);
        }
    }
    out
}

pub(super) fn is_ws(ch: char) -> bool {
    matches!(
        ch,
        ' ' | '\t'
            | '\n'
            | '\r'
            | '\u{000b}'
            | '\u{000c}'
            | '\u{0085}'
            | '\u{200e}'
            | '\u{200f}'
            | '\u{2028}'
            | '\u{2029}'
    )
}

pub(super) fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic() || (!ch.is_ascii() && ch.is_alphabetic())
}

pub(super) fn is_ident_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric() || (!ch.is_ascii() && ch.is_alphanumeric())
}
