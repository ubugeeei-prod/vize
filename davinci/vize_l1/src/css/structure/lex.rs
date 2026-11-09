//! What a byte starts in code: a comment, an opaque run (a string, an
//! unquoted `url(…)`, an escape), an interpolation, or nothing special.

use super::{Interpolation, Lexis};

pub(super) enum Lexeme {
    Comment {
        end: usize,
        line: bool,
        terminated: bool,
    },
    /// Code whose bytes never open, close or end anything. Interpolation
    /// inside a string only changes the string.
    Opaque {
        end: usize,
        terminated: bool,
    },
    Interpolation {
        end: usize,
        terminated: bool,
    },
    Byte,
}

/// Read the lexeme at `index`. `braced` says whether `{` can open a block.
/// In indented syntax strings, URLs and comments end with their line.
pub(super) fn lex(bytes: &[u8], index: usize, lexis: &Lexis, braced: bool) -> Lexeme {
    let Some(&byte) = bytes.get(index) else {
        return Lexeme::Byte;
    };
    if !matches!(
        byte,
        b'/' | b'\'' | b'"' | b'\\' | b'#' | b'@' | b'{' | b'u' | b'U'
    ) {
        return Lexeme::Byte;
    }
    let next = bytes.get(index + 1).copied();
    let line_end = || {
        bytes
            .get(index..)
            .and_then(|rest| rest.iter().position(|&b| b == b'\n'))
            .map_or(bytes.len(), |offset| index + offset)
    };
    match (byte, next) {
        (b'/', Some(b'*')) => {
            let limit = if lexis.indented_comments {
                line_end()
            } else {
                bytes.len()
            };
            match find(bytes, index + 2, limit, b"*/") {
                Some(close) => Lexeme::Comment {
                    end: close + 2,
                    line: false,
                    terminated: true,
                },
                None => Lexeme::Comment {
                    end: limit,
                    line: false,
                    terminated: false,
                },
            }
        }
        (b'/', Some(b'/')) if lexis.line_comments => Lexeme::Comment {
            end: line_end(),
            line: true,
            terminated: true,
        },
        (b'\'' | b'"', _) => {
            let limit = if lexis.indented {
                line_end()
            } else {
                bytes.len()
            };
            string(bytes, index, limit, lexis)
        }
        (b'\\', _) => Lexeme::Opaque {
            end: char_end(bytes, index + 1),
            terminated: true,
        },
        _ if opens_interpolation(bytes, index, lexis, braced) => {
            let (end, terminated) = interpolation(bytes, index, lexis);
            Lexeme::Interpolation { end, terminated }
        }
        (b'u' | b'U', _) => url(
            bytes,
            index,
            if lexis.indented {
                line_end()
            } else {
                bytes.len()
            },
        ),
        _ => Lexeme::Byte,
    }
}

/// The end of the run from `index` of bytes that lex as plain code and are
/// neither whitespace nor structure (`{ } ; ( ) [ ] :`), so a reader can
/// take the run in one step.
pub(super) fn plain_run(bytes: &[u8], index: usize) -> usize {
    let mut end = index;
    while let Some(&byte) = bytes.get(end) {
        if is_plain_byte(byte) {
            end += 1;
            continue;
        }
        let plain = match byte {
            b'/' => !matches!(bytes.get(end + 1), Some(b'*' | b'/')),
            b'#' | b'@' => bytes.get(end + 1) != Some(&b'{'),
            // Only a `u` that starts a name can open `url(`.
            b'u' | b'U' => end > index && bytes.get(end - 1).is_some_and(|&b| is_name_byte(b)),
            _ => false,
        };
        if !plain {
            break;
        }
        end += 1;
    }
    end
}

/// Bytes that are plain code wherever they appear: not whitespace, not
/// structure, and never the start of a comment, string, escape,
/// interpolation or `url(`.
fn is_plain_byte(byte: u8) -> bool {
    BYTE_CLASSES.get(usize::from(byte)).copied().unwrap_or(0) & PLAIN != 0
}

/// Bytes that continue a name: ASCII letters and digits, `-`, `_`, and
/// every byte of a non-ASCII character.
pub(super) fn is_name_byte(byte: u8) -> bool {
    BYTE_CLASSES.get(usize::from(byte)).copied().unwrap_or(0) & NAME != 0
}

const PLAIN: u8 = 1;
const NAME: u8 = 2;

/// One lookup per byte for the readers' hottest questions.
#[expect(
    clippy::indexing_slicing,
    reason = "evaluated at compile time: an index past 255 fails the build"
)]
const BYTE_CLASSES: [u8; 256] = {
    let mut table = [0u8; 256];
    let mut index = 0usize;
    while index < 256 {
        let byte = index as u8;
        let structural = byte.is_ascii_whitespace()
            || matches!(
                byte,
                b'{' | b'}'
                    | b';'
                    | b'('
                    | b')'
                    | b'['
                    | b']'
                    | b':'
                    | b'\''
                    | b'"'
                    | b'\\'
                    | b'/'
                    | b'#'
                    | b'@'
                    | b'u'
                    | b'U'
            );
        let name = byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_' || !byte.is_ascii();
        table[index] = if structural { 0 } else { PLAIN } | if name { NAME } else { 0 };
        index += 1;
    }
    table
};

/// Whether `index` starts an interpolation in this language.
pub(super) fn opens_interpolation(bytes: &[u8], index: usize, lexis: &Lexis, braced: bool) -> bool {
    let next = bytes.get(index + 1).copied();
    match (lexis.interpolation, bytes.get(index).copied()) {
        (Interpolation::Hash, Some(b'#')) | (Interpolation::At, Some(b'@')) => next == Some(b'{'),
        (Interpolation::Brace, Some(b'{')) => {
            !braced
                || index
                    .checked_sub(1)
                    .and_then(|previous| bytes.get(previous))
                    .is_some_and(|&previous| is_name_byte(previous))
        }
        _ => false,
    }
}

/// The end of the interpolation at `index`, after its matching `}`.
fn interpolation(bytes: &[u8], index: usize, lexis: &Lexis) -> (usize, bool) {
    let mut cursor = index + usize::from(lexis.interpolation != Interpolation::Brace) + 1;
    let mut depth = 1usize;
    while let Some(&byte) = bytes.get(cursor) {
        match byte {
            b'\'' | b'"' => {
                cursor = match string(bytes, cursor, bytes.len(), lexis) {
                    Lexeme::Opaque { end, .. } => end,
                    _ => cursor + 1,
                };
                continue;
            }
            b'\\' => cursor += 1,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return (cursor + 1, true);
                }
            }
            _ => {}
        }
        cursor += 1;
    }
    (bytes.len(), false)
}

fn string(bytes: &[u8], index: usize, limit: usize, lexis: &Lexis) -> Lexeme {
    let quote = bytes.get(index).copied();
    let mut cursor = index + 1;
    while cursor < limit {
        let Some(&byte) = bytes.get(cursor) else {
            break;
        };
        if byte == b'\\' {
            cursor = char_end(bytes, cursor + 1);
            continue;
        }
        if Some(byte) == quote {
            return Lexeme::Opaque {
                end: cursor + 1,
                terminated: true,
            };
        }
        if matches!(lexis.interpolation, Interpolation::Hash | Interpolation::At)
            && opens_interpolation(bytes, cursor, lexis, true)
        {
            cursor = interpolation(bytes, cursor, lexis).0;
            continue;
        }
        cursor += 1;
    }
    Lexeme::Opaque {
        end: limit.min(bytes.len()),
        terminated: false,
    }
}

/// An unquoted `url(…)` payload is opaque: `//`, braces and `;` in it are data.
fn url(bytes: &[u8], index: usize, limit: usize) -> Lexeme {
    let starts_name = index
        .checked_sub(1)
        .and_then(|previous| bytes.get(previous))
        .is_none_or(|&previous| !is_name_byte(previous));
    let named = bytes
        .get(index..index + 4)
        .is_some_and(|name| name.eq_ignore_ascii_case(b"url("));
    if !starts_name || !named {
        return Lexeme::Byte;
    }
    let mut cursor = index + 4;
    while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
        cursor += 1;
    }
    if matches!(bytes.get(cursor), Some(b'\'' | b'"')) {
        return Lexeme::Byte;
    }
    while cursor < limit {
        match bytes.get(cursor) {
            Some(b')') => {
                return Lexeme::Opaque {
                    end: cursor + 1,
                    terminated: true,
                };
            }
            Some(b'\\') => cursor = char_end(bytes, cursor + 1),
            Some(_) => cursor += 1,
            None => break,
        }
    }
    Lexeme::Opaque {
        end: limit.min(bytes.len()),
        terminated: false,
    }
}

fn find(bytes: &[u8], from: usize, limit: usize, needle: &[u8]) -> Option<usize> {
    bytes
        .get(from..limit)?
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|offset| from + offset)
}

/// The end of the character starting at `index`, so spans stay on boundaries.
pub(super) fn char_end(bytes: &[u8], index: usize) -> usize {
    let mut end = index + 1;
    while bytes.get(end).is_some_and(|&byte| byte & 0xC0 == 0x80) {
        end += 1;
    }
    end.min(bytes.len())
}
