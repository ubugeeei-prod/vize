//! Stylus: indented or braced rules, colonless declarations, `{}`
//! interpolation, and its own statements (assignments, calls, control flow).

use vize_l0::String;

use super::structure::{Construct, Interpolation, Lexis};

/// Stylus reads braces when any line opens one, indentation otherwise.
pub(super) fn lexis(source: &str) -> Lexis {
    Lexis {
        line_comments: true,
        interpolation: Interpolation::Brace,
        indented: !is_braced(source),
        indented_comments: false,
        colonless_declarations: true,
        construct: stylus_construct,
    }
}

/// Stylus accepts CSS braces. Outside comments, strings and unquoted
/// `url(…)` payloads, a block brace ends a line (`.a {`) or, in a one-line
/// rule (`.a { color: red; }`), follows whitespace and its `}` encloses a `;`
/// or a `: ` (in a string too: `.a {content:"a;b"}`); an interpolation
/// (`.item-{i}`, `{prop}: 1`) does neither. The choice is per block, so a
/// block that mixes braced and indented rules is read one way only.
pub(super) fn is_braced(source: &str) -> bool {
    let code = code_lines(source);
    // `code` keeps every byte offset of `source`, so the same range of a
    // line of each is the same authored bytes.
    source.lines().zip(code.lines()).any(|(authored, line)| {
        if line.trim_end().ends_with('{') {
            return true;
        }
        let bytes = line.as_bytes();
        bytes.iter().enumerate().any(|(at, &byte)| {
            byte == b'{'
                && at > 0
                && bytes.get(at - 1).is_some_and(u8::is_ascii_whitespace)
                && line
                    .get(at + 1..)
                    .and_then(|rest| rest.find('}'))
                    .and_then(|close| authored.get(at + 1..at + 1 + close))
                    .is_some_and(|inner| inner.contains(';') || inner.contains(": "))
        })
    })
}

/// `source` with comments, string contents and unquoted `url(…)` payloads
/// blanked byte for byte, line breaks kept, so every offset is the authored
/// one. A payload is opaque as the structure lexer reads it: `//`, braces
/// and `;` in it are data.
fn code_lines(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut code = bytes.to_vec();
    let mut blank = |from: usize, to: usize| {
        for byte in code.get_mut(from..to).into_iter().flatten() {
            if *byte != b'\n' {
                *byte = b' ';
            }
        }
    };
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        index = match byte {
            // An escaped character is opaque, as the structure lexer reads
            // it: `.quoted\"name` opens no string.
            b'\\' => {
                let end = char_end(bytes, index + 1);
                blank(index + 1, end);
                end
            }
            b'"' | b'\'' => {
                let close = string_close(bytes, index);
                blank(index + 1, close);
                close + 1
            }
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                let end = find(bytes, index, b"\n").unwrap_or(bytes.len());
                blank(index, end);
                end
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                let end = find(bytes, index + 2, b"*/").map_or(bytes.len(), |at| at + 2);
                blank(index, end);
                end
            }
            b'u' | b'U' => match url_payload(bytes, index) {
                Some((from, close)) => {
                    blank(from, close);
                    close + 1
                }
                None => index + 1,
            },
            _ => index + 1,
        };
    }
    // Blanks start and end at ASCII bytes, so whole characters stay whole.
    core::str::from_utf8(&code)
        .map(String::from)
        .unwrap_or_default()
}

/// The index of the quote closing the string opened at `open`, or the end.
fn string_close(bytes: &[u8], open: usize) -> usize {
    let quote = bytes.get(open).copied();
    let mut cursor = open + 1;
    while let Some(&byte) = bytes.get(cursor) {
        if byte == b'\\' {
            cursor = char_end(bytes, cursor + 1);
            continue;
        }
        if Some(byte) == quote {
            return cursor;
        }
        cursor += 1;
    }
    bytes.len()
}

/// The payload start and the index of the `)` (or the end) of an unquoted
/// `url(` that starts at `index`.
fn url_payload(bytes: &[u8], index: usize) -> Option<(usize, usize)> {
    let starts_name = index
        .checked_sub(1)
        .and_then(|previous| bytes.get(previous))
        .is_none_or(|&previous| {
            !(previous.is_ascii_alphanumeric()
                || matches!(previous, b'-' | b'_')
                || !previous.is_ascii())
        });
    let named = bytes
        .get(index..index + 4)
        .is_some_and(|name| name.eq_ignore_ascii_case(b"url("));
    if !starts_name || !named {
        return None;
    }
    let mut from = index + 4;
    while bytes.get(from).is_some_and(u8::is_ascii_whitespace) {
        from += 1;
    }
    if matches!(bytes.get(from), Some(b'\'' | b'"')) {
        return None;
    }
    let mut cursor = from;
    while let Some(&byte) = bytes.get(cursor) {
        match byte {
            b')' => return Some((from, cursor)),
            b'\\' => cursor = char_end(bytes, cursor + 1),
            _ => cursor += 1,
        }
    }
    Some((from, bytes.len()))
}

/// The end of the character starting at `index` (the end of `bytes` past it).
fn char_end(bytes: &[u8], index: usize) -> usize {
    if index >= bytes.len() {
        return bytes.len();
    }
    let mut end = index + 1;
    while bytes.get(end).is_some_and(|&byte| byte & 0xC0 == 0x80) {
        end += 1;
    }
    end
}

fn find(bytes: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    bytes
        .get(from..)?
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|offset| from + offset)
}

/// Control flow, block mixins (`+name()`), calls and function definitions
/// (`name(…)`) and assignments (`name = value`) are Stylus, not CSS.
fn stylus_construct(text: &str) -> Construct {
    let keyword = text
        .split(|c: char| c.is_ascii_whitespace() || c == '(')
        .next()
        .unwrap_or_default();
    let name_end = text
        .find(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '$')))
        .unwrap_or(text.len());
    let rest = text.get(name_end..).unwrap_or_default().trim_start();
    let assigns = name_end > 0
        && (rest.starts_with('=') && !rest.starts_with("==")
            || ["?=", ":=", "+=", "-=", "*=", "/="]
                .iter()
                .any(|operator| rest.starts_with(operator)));
    if text.starts_with('+')
        || matches!(keyword, "if" | "unless" | "for" | "else" | "return")
        || starts_call(text)
        || assigns
    {
        Construct::Other
    } else {
        Construct::Shared
    }
}

/// `name(`: a Stylus mixin or function, defined when it opens a block.
pub(super) fn starts_call(text: &str) -> bool {
    text.find('(')
        .and_then(|open| text.get(..open))
        .is_some_and(|name| {
            !name.is_empty()
                && name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        })
}
