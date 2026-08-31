//! HTML entity decoding for JS string emission.
//!
//! This mirrors `vize_armature`'s tokenizer entity rules so S2 text and static
//! prop emission see the same decoded content the shipped DOM lane lowers.

use core::cmp::min;
use core::num::IntErrorKind;

use htmlize::{Context, ENTITIES, ENTITY_MAX_LENGTH, ENTITY_MIN_LENGTH};
use vize_s0::String;

pub(super) fn decode_html_entities(source: &str) -> String {
    if !source.contains('&') {
        return String::from(source);
    }

    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut index = 0usize;
    while index < bytes.len() {
        if let Some((ch, consumed)) = decode_escaped_numeric_parenthesis(&bytes[index..]) {
            out.push(ch);
            index += consumed;
            continue;
        }
        if let Some((ch, consumed)) = try_decode_entity(&bytes[index..], Context::General) {
            out.push(ch);
            index += consumed;
            continue;
        }

        let ch = source[index..].chars().next().unwrap_or('\u{FFFD}');
        out.push(ch);
        index += ch.len_utf8();
    }
    out
}

fn decode_escaped_numeric_parenthesis(input: &[u8]) -> Option<(char, usize)> {
    const OPEN: &[u8] = b"&amp;#40;";
    const OPEN_PADDED: &[u8] = b"&amp;#040;";
    const CLOSE: &[u8] = b"&amp;#41;";
    const CLOSE_PADDED: &[u8] = b"&amp;#041;";
    if input.starts_with(OPEN) {
        Some(('(', OPEN.len()))
    } else if input.starts_with(OPEN_PADDED) {
        Some(('(', OPEN_PADDED.len()))
    } else if input.starts_with(CLOSE) {
        Some((')', CLOSE.len()))
    } else if input.starts_with(CLOSE_PADDED) {
        Some((')', CLOSE_PADDED.len()))
    } else {
        None
    }
}

fn try_decode_entity(input: &[u8], context: Context) -> Option<(char, usize)> {
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
    core::str::from_utf8(expansion).ok()?.chars().next()
}

fn decode_named_entity(input: &[u8], context: Context) -> Option<(char, usize)> {
    let mut index = 1usize;
    let mut steps = 0usize;
    while steps < ENTITY_MAX_LENGTH - 1 && index < input.len() {
        if input[index].is_ascii_alphanumeric() {
            index += 1;
            steps += 1;
        } else {
            break;
        }
    }

    let mut consumed_end = index;
    match input.get(index).copied() {
        Some(b';') => consumed_end = index + 1,
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

    let mut position = 2usize;
    let number = match input.get(position).copied() {
        Some(b'x' | b'X') => {
            position += 1;
            let start = position;
            while position < input.len() && input[position].is_ascii_hexdigit() {
                position += 1;
            }
            let hex = &input[start..position];
            if hex.is_empty() {
                return None;
            }
            u32::from_str_radix(core::str::from_utf8(hex).ok()?, 16)
        }
        Some(c) if c.is_ascii_digit() => {
            let start = position;
            while position < input.len() && input[position].is_ascii_digit() {
                position += 1;
            }
            let dec = &input[start..position];
            if dec.is_empty() {
                return None;
            }
            core::str::from_utf8(dec).ok()?.parse::<u32>()
        }
        _ => return None,
    };

    let mut end = position;
    if input.get(position) == Some(&b';') {
        end = position + 1;
    }

    let ch = match number {
        Ok(n) => correct_numeric_entity(n),
        Err(error) if *error.kind() == IntErrorKind::PosOverflow => '\u{FFFD}',
        Err(_) => return None,
    };
    Some((ch, end))
}

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
        scalar => char::from_u32(scalar).unwrap_or('\u{FFFD}'),
    }
}

#[cfg(test)]
mod tests {
    use super::decode_html_entities;

    #[test]
    fn decodes_named_entities_like_text_tokenization() {
        assert_eq!(decode_html_entities("&times;"), "\u{00d7}");
        assert_eq!(decode_html_entities("&timesX"), "\u{00d7}X");
        assert_eq!(decode_html_entities("&&amp;"), "&&");
    }

    #[test]
    fn applies_html_numeric_correction_table() {
        assert_eq!(decode_html_entities("&#128;"), "\u{20ac}");
        assert_eq!(decode_html_entities("&#55296;"), "\u{fffd}");
    }
}
