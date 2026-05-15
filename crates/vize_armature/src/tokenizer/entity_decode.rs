//! Decode at most one HTML entity from the start of a byte slice (`&name;`, `&#...;`, …).
//! Rules align with `htmlize::unescape_bytes_in` (WHATWG), using the same `ENTITIES` map.

use std::cmp::min;
use std::num::IntErrorKind;

use htmlize::{Context, ENTITIES, ENTITY_MAX_LENGTH, ENTITY_MIN_LENGTH};

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
            std::str::from_utf8(dec).ok()?.parse::<u32>()
        }
        _ => return None,
    };

    let mut end = pos;
    if input.get(pos) == Some(&b';') {
        end = pos + 1;
    }

    let ch = match number {
        Ok(n) => correct_numeric_entity(n),
        Err(e) if *e.kind() == IntErrorKind::PosOverflow => '\u{FFFD}',
        Err(_) => return None,
    };
    Some((ch, end))
}

/// <https://html.spec.whatwg.org/multipage/parsing.html#numeric-character-reference-end-state>
#[allow(clippy::match_same_arms)]
fn correct_numeric_entity(number: u32) -> char {
    match number {
        0x00 => '\u{FFFD}',
        0x11_0000.. => '\u{FFFD}',
        0xD800..=0xDFFF => '\u{FFFD}',
        0x80 => '\u{20AC}',
        0x82 => '\u{201A}',
        0x83 => '\u{0192}',
        0x84 => '\u{201E}',
        0x85 => '\u{2026}',
        0x86 => '\u{2020}',
        0x87 => '\u{2021}',
        0x88 => '\u{02C6}',
        0x89 => '\u{2030}',
        0x8A => '\u{0160}',
        0x8B => '\u{2039}',
        0x8C => '\u{0152}',
        0x8E => '\u{017D}',
        0x91 => '\u{2018}',
        0x92 => '\u{2019}',
        0x93 => '\u{201C}',
        0x94 => '\u{201D}',
        0x95 => '\u{2022}',
        0x96 => '\u{2013}',
        0x97 => '\u{2014}',
        0x98 => '\u{02DC}',
        0x99 => '\u{2122}',
        0x9A => '\u{0161}',
        0x9B => '\u{203A}',
        0x9C => '\u{0153}',
        0x9E => '\u{017E}',
        0x9F => '\u{0178}',
        c => char::from_u32(c).unwrap_or('\u{FFFD}'),
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
    fn numeric_surrogate_replaced() {
        let s = b"&#55296;";
        let (c, n) = try_decode_entity(s, Context::General).unwrap();
        assert_eq!(c, '\u{FFFD}');
        assert_eq!(n, 8);
    }

    #[test]
    fn numeric_windows_1252_mapping() {
        let s = b"&#128;";
        let (c, n) = try_decode_entity(s, Context::General).unwrap();
        assert_eq!(c, '\u{20AC}');
        assert_eq!(n, 6);
    }

    #[test]
    fn bare_ampersand_is_none() {
        assert!(try_decode_entity(b"&", Context::General).is_none());
        assert!(try_decode_entity(b"&@", Context::General).is_none());
    }
}
