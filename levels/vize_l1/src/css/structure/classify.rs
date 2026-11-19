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
        let name = trimmed(bytes, 0, colon);
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
    if starts
        && name
            .iter()
            .skip(offset)
            .all(|&byte| is_name_byte(byte) || byte == b'\\')
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
        let valid = if index == start + offset {
            byte.is_ascii_alphabetic() || matches!(byte, b'-' | b'_' | b'\\') || !byte.is_ascii()
        } else {
            is_name_byte(byte) || byte == b'\\'
        };
        if !valid {
            return false;
        }
        index += 1;
    }
    index == end
}
