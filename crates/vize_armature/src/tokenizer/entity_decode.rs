//! Decode at most one HTML entity from the start of a byte slice (`&name;`, `&#...;`, …).
//! Rules align with `htmlize::unescape_bytes_in` (WHATWG), using the same `ENTITIES` map.

use std::borrow::Cow;
use std::cmp::min;
use std::num::IntErrorKind;

use htmlize::{Context, ENTITIES, ENTITY_MAX_LENGTH, ENTITY_MIN_LENGTH, REPLACEMENT_CHAR_BYTES};

/// If `input` starts with a valid entity, returns the first decoded scalar and the number of
/// bytes consumed (including `&` and an optional `;`). Otherwise `None` so the tokenizer can
/// emit `&` as literal text.
pub(super) fn try_decode_entity(input: &[u8], context: Context) -> Option<(char, usize)> {
    if input.first() != Some(&b'&') {
        return None;
    }
    if input.get(1) == Some(&b'#') {
        decode_numeric_entity(input)
    } else {
        decode_named_entity(input, context)
    }
}

fn first_scalar(expansion: &[u8]) -> Option<char> {
    std::str::from_utf8(expansion).ok()?.chars().next()
}

fn decode_named_entity(input: &[u8], context: Context) -> Option<(char, usize)> {
    let mut j = 1usize;
    let mut steps = 0usize;
    while steps < ENTITY_MAX_LENGTH - 1 && j < input.len() {
        if input[j].is_ascii_alphanumeric() {
            j += 1;
            steps += 1;
        } else {
            break;
        }
    }

    let mut consumed_end = j;
    match input.get(j).copied() {
        Some(b';') => consumed_end = j + 1,
        Some(b'=') if context == Context::Attribute => return None,
        _ => {}
    }

    if context == Context::Attribute {
        let candidate = &input[..consumed_end];
        if candidate.len() < ENTITY_MIN_LENGTH {
            return None;
        }
        let expansion = ENTITIES.get(candidate)?;
        let ch = first_scalar(expansion)?;
        return Some((ch, consumed_end));
    }

    let max_len = min(consumed_end, ENTITY_MAX_LENGTH);
    for check_len in (ENTITY_MIN_LENGTH..=max_len).rev() {
        if let Some(expansion) = ENTITIES.get(&input[..check_len]) {
            let ch = first_scalar(expansion)?;
            return Some((ch, check_len));
        }
    }
    None
}

fn decode_numeric_entity(input: &[u8]) -> Option<(char, usize)> {
    if input.len() < 3 || input[0] != b'&' || input[1] != b'#' {
        return None;
    }

    let mut pos = 2usize;
    let number = match input.get(pos).copied() {
        Some(b'x' | b'X') => {
            pos += 1;
            let start = pos;
            while pos < input.len() && input[pos].is_ascii_hexdigit() {
                pos += 1;
            }
            let hex = &input[start..pos];
            if hex.is_empty() {
                return None;
            }
            u32::from_str_radix(std::str::from_utf8(hex).ok()?, 16)
        }
        Some(c) if c.is_ascii_digit() => {
            let start = pos;
            while pos < input.len() && input[pos].is_ascii_digit() {
                pos += 1;
            }
            let dec = &input[start..pos];
            if dec.is_empty() {
                return None;
            }
            u32::from_str_radix(std::str::from_utf8(dec).ok()?, 10)
        }
        _ => return None,
    };

    let mut end = pos;
    if input.get(pos) == Some(&b';') {
        end = pos + 1;
    }

    let cow = match number {
        Ok(n) => correct_numeric_entity(n),
        Err(e) if *e.kind() == IntErrorKind::PosOverflow => Cow::Borrowed(REPLACEMENT_CHAR_BYTES),
        Err(_) => return None,
    };
    let ch = first_scalar(cow.as_ref())?;
    Some((ch, end))
}

/// <https://html.spec.whatwg.org/multipage/parsing.html#numeric-character-reference-end-state>
#[allow(clippy::match_same_arms)]
fn correct_numeric_entity(number: u32) -> Cow<'static, [u8]> {
    match number {
        0x00 => Cow::Borrowed(REPLACEMENT_CHAR_BYTES),
        0x11_0000.. => Cow::Borrowed(REPLACEMENT_CHAR_BYTES),
        0xD800..=0xDFFF => Cow::Borrowed(REPLACEMENT_CHAR_BYTES),
        0x80 => Cow::Borrowed("\u{20AC}".as_bytes()),
        0x82 => Cow::Borrowed("\u{201A}".as_bytes()),
        0x83 => Cow::Borrowed("\u{0192}".as_bytes()),
        0x84 => Cow::Borrowed("\u{201E}".as_bytes()),
        0x85 => Cow::Borrowed("\u{2026}".as_bytes()),
        0x86 => Cow::Borrowed("\u{2020}".as_bytes()),
        0x87 => Cow::Borrowed("\u{2021}".as_bytes()),
        0x88 => Cow::Borrowed("\u{02C6}".as_bytes()),
        0x89 => Cow::Borrowed("\u{2030}".as_bytes()),
        0x8A => Cow::Borrowed("\u{0160}".as_bytes()),
        0x8B => Cow::Borrowed("\u{2039}".as_bytes()),
        0x8C => Cow::Borrowed("\u{0152}".as_bytes()),
        0x8E => Cow::Borrowed("\u{017D}".as_bytes()),
        0x91 => Cow::Borrowed("\u{2018}".as_bytes()),
        0x92 => Cow::Borrowed("\u{2019}".as_bytes()),
        0x93 => Cow::Borrowed("\u{201C}".as_bytes()),
        0x94 => Cow::Borrowed("\u{201D}".as_bytes()),
        0x95 => Cow::Borrowed("\u{2022}".as_bytes()),
        0x96 => Cow::Borrowed("\u{2013}".as_bytes()),
        0x97 => Cow::Borrowed("\u{2014}".as_bytes()),
        0x98 => Cow::Borrowed("\u{02DC}".as_bytes()),
        0x99 => Cow::Borrowed("\u{2122}".as_bytes()),
        0x9A => Cow::Borrowed("\u{0161}".as_bytes()),
        0x9B => Cow::Borrowed("\u{203A}".as_bytes()),
        0x9C => Cow::Borrowed("\u{0153}".as_bytes()),
        0x9E => Cow::Borrowed("\u{017E}".as_bytes()),
        0x9F => Cow::Borrowed("\u{0178}".as_bytes()),
        c => char::from_u32(c)
            .map(|c| Cow::Owned(c.to_string().into_bytes()))
            .unwrap_or_else(|| Cow::Borrowed(REPLACEMENT_CHAR_BYTES)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn general_named_semicolon() {
        let s = b"&amp;rest";
        let (c, n) = try_decode_entity(s, Context::General).unwrap();
        assert_eq!(c, '&');
        assert_eq!(n, 5);
        assert_eq!(&s[n..], b"rest");
    }

    #[test]
    fn general_named_no_semicolon_longest() {
        let s = b"&timesX";
        let (c, n) = try_decode_entity(s, Context::General).unwrap();
        assert_eq!(c, '\u{00d7}');
        assert_eq!(n, 6);
    }

    #[test]
    fn attribute_times_x_not_entity() {
        assert!(try_decode_entity(b"&timesX", Context::Attribute).is_none());
    }

    #[test]
    fn numeric_dec() {
        let s = b"&#38;z";
        let (c, n) = try_decode_entity(s, Context::General).unwrap();
        assert_eq!(c, '&');
        assert_eq!(n, 5);
    }

    /// `&` + `&amp;`: leading `&` alone is not a valid entity (next byte is `&`).
    #[test]
    fn double_ampersand_amp_from_start_is_none() {
        assert!(try_decode_entity(b"&&amp;", Context::General).is_none());
        assert!(try_decode_entity(b"&&amp;", Context::Attribute).is_none());
    }

    /// Same bytes as tokenizer would pass after consuming the first literal `&`.
    #[test]
    fn double_ampersand_decode_second_reference() {
        let s = b"&&amp;";
        let (c, n) = try_decode_entity(&s[1..], Context::General).unwrap();
        assert_eq!(c, '&');
        assert_eq!(n, 5);
        assert_eq!(1 + n, s.len());
    }

    #[test]
    fn hex_numeric() {
        let s = b"&#x26;y";
        let (c, n) = try_decode_entity(s, Context::General).unwrap();
        assert_eq!(c, '&');
        assert_eq!(n, 6);
        assert_eq!(&s[n..], b"y");
    }

    #[test]
    fn attribute_named_with_semicolon() {
        let s = b"&lt;";
        let (c, n) = try_decode_entity(s, Context::Attribute).unwrap();
        assert_eq!(c, '<');
        assert_eq!(n, 4);
    }

    #[test]
    fn bare_ampersand_is_none() {
        assert!(try_decode_entity(b"&", Context::General).is_none());
        assert!(try_decode_entity(b"&@", Context::General).is_none());
    }
}
