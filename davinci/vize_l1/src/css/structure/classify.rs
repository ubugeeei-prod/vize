//! The shared grammar of one prelude: at-rule, declaration, rule or other.
//!
//! A block prelude `name: value` is a declaration (a nested property) only
//! when the colon is followed by whitespace or nothing, as Sass decides it;
//! `a:hover` stays a selector.

use super::lex::{Lexeme, is_name_byte, lex, opens_interpolation};
use super::{Construct, Lexis};

/// Offsets relative to the prelude text.
pub(super) enum Shape {
    Rule,
    AtRule {
        name: (usize, usize),
    },
    Declaration {
        name: (usize, usize),
        value: (usize, usize),
    },
    Other,
}

/// `colon` is the first `:` outside strings, comments, interpolation and
/// brackets, as the reader found it.
pub(super) fn classify(text: &str, colon: Option<usize>, block: bool, lexis: &Lexis) -> Shape {
    match (lexis.construct)(text) {
        Construct::Sigil => return Shape::AtRule { name: (0, 1) },
        Construct::Other => return Shape::Other,
        Construct::Shared => {}
    }
    let bytes = text.as_bytes();
    if bytes.first() == Some(&b'@') && !opens_interpolation(bytes, 0, lexis, true) {
        let end = 1 + bytes
            .get(1..)
            .unwrap_or_default()
            .iter()
            .take_while(|&&byte| is_name_byte(byte))
            .count();
        // Less variables and detached rulesets: `@gap: 4px`, `@r: { … }`.
        if end > 1 && bytes.get(end) == Some(&b':') {
            return declaration(text, (0, end), end + 1);
        }
        return Shape::AtRule { name: (1, end) };
    }
    if let Some(colon) = colon {
        let name = name_before_trailing_comments(bytes, trimmed(bytes, 0, colon), lexis);
        let after = bytes.get(colon + 1);
        if is_property_name(bytes, name, lexis)
            && (!block || after.is_none_or(u8::is_ascii_whitespace))
        {
            return declaration(text, name, colon + 1);
        }
    }
    if lexis.colonless_declarations && !block {
        let name_end = bytes
            .iter()
            .position(u8::is_ascii_whitespace)
            .unwrap_or(bytes.len());
        if name_end < bytes.len() && is_property_name(bytes, (0, name_end), lexis) {
            return declaration(text, (0, name_end), name_end);
        }
    }
    if block { Shape::Rule } else { Shape::Other }
}

/// Keep a property identifier's authored span when only whitespace and block
/// comments separate it from the colon (`color/**/: red`). Comments are not
/// identifier bytes, and invalid text after the first comment must not be
/// joined into a declaration name.
fn name_before_trailing_comments(
    bytes: &[u8],
    original: (usize, usize),
    lexis: &Lexis,
) -> (usize, usize) {
    let (start, end) = original;
    let mut cursor = start;
    while cursor < end {
        if bytes.get(cursor) == Some(&b'\\') {
            let Some(escape_end) = css_escape_end(bytes, cursor, end) else {
                return original;
            };
            cursor = escape_end;
            continue;
        }
        if bytes.get(cursor).is_some_and(u8::is_ascii_whitespace)
            || matches!(lex(bytes, cursor, lexis, true), Lexeme::Comment { .. })
        {
            break;
        }
        cursor = match lex(bytes, cursor, lexis, true) {
            Lexeme::Opaque { end, .. } | Lexeme::Interpolation { end, .. } => end,
            _ => cursor + 1,
        };
    }
    let name = (start, cursor);
    if !is_property_name(bytes, name, lexis) {
        return original;
    }
    while cursor < end {
        if bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
            continue;
        }
        match lex(bytes, cursor, lexis, true) {
            Lexeme::Comment { end, .. } => cursor = end,
            _ => return original,
        }
    }
    name
}

fn declaration(text: &str, name: (usize, usize), value_start: usize) -> Shape {
    Shape::Declaration {
        name,
        value: trimmed(text.as_bytes(), value_start, text.len()),
    }
}

fn trimmed(bytes: &[u8], mut start: usize, mut end: usize) -> (usize, usize) {
    while start < end && bytes.get(start).is_some_and(u8::is_ascii_whitespace) {
        start += 1;
    }
    while end > start && bytes.get(end - 1).is_some_and(u8::is_ascii_whitespace) {
        end -= 1;
    }
    (start, end)
}

/// A property name: `color`, `--custom`, `$variable`, `*zoom`, and names
/// built from interpolation (`#{$side}-margin`).
fn is_property_name(bytes: &[u8], (start, end): (usize, usize), lexis: &Lexis) -> bool {
    let name = bytes.get(start..end).unwrap_or_default();
    let offset = usize::from(matches!(name.first(), Some(b'$' | b'*')));
    let Some(&first) = name.get(offset) else {
        return false;
    };
    let starts =
        first.is_ascii_alphabetic() || matches!(first, b'-' | b'_' | b'\\') || !first.is_ascii();
    if starts && !name.contains(&b'\\') && name.iter().skip(offset).all(|&byte| is_name_byte(byte))
    {
        return true;
    }
    // Interpolation may build any part of the name.
    let mut index = start + offset;
    while index < end {
        let Some(&byte) = bytes.get(index) else {
            return false;
        };
        if opens_interpolation(bytes, index, lexis, true) {
            match lex(bytes, index, lexis, true) {
                Lexeme::Interpolation { end, .. } => index = end,
                _ => return false,
            }
            continue;
        }
        if byte == b'\\' {
            let Some(end) = css_escape_end(bytes, index, end) else {
                return false;
            };
            index = end;
            continue;
        }
        let valid = if index == start + offset {
            byte.is_ascii_alphabetic() || matches!(byte, b'-' | b'_') || !byte.is_ascii()
        } else {
            is_name_byte(byte)
        };
        if !valid {
            return false;
        }
        index += 1;
    }
    index == end
}

/// End of one valid CSS escape. A hex escape consumes one to six digits and
/// one optional whitespace terminator; a CRLF pair is one terminator.
fn css_escape_end(bytes: &[u8], slash: usize, limit: usize) -> Option<usize> {
    let first = slash.checked_add(1)?;
    let byte = *bytes.get(first..limit)?.first()?;
    if byte.is_ascii_hexdigit() {
        let mut end = first;
        while end < limit && end - first < 6 && bytes.get(end).is_some_and(u8::is_ascii_hexdigit) {
            end += 1;
        }
        if bytes.get(end).is_some_and(u8::is_ascii_whitespace) {
            if bytes.get(end) == Some(&b'\r') && bytes.get(end + 1) == Some(&b'\n') {
                return Some(end + 2);
            }
            end += 1;
        }
        return Some(end);
    }
    if matches!(byte, b'\n' | b'\r' | 0x0c) {
        return None;
    }
    let width = if byte < 0x80 {
        1
    } else if byte < 0xe0 {
        2
    } else if byte < 0xf0 {
        3
    } else {
        4
    };
    (first + width <= limit).then_some(first + width)
}
