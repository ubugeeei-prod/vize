//! Low-level byte scanners for the expression nesting guard.
//!
//! Each helper advances past a lexical construct OXC's lexer skips or consumes
//! as a unit (string and template literals, comments, regex literals,
//! identifiers, numbers), so the depth scanner in the parent module never
//! counts brackets or type angles hidden inside them. The byte scanner must
//! stop exactly where the lexer stops, or hidden brackets slip past the guard
//! while OXC still recurses into the overflow path.

use oxc_syntax::identifier::is_identifier_start;

pub fn skip_quoted(bytes: &[u8], mut i: usize, quote: u8) -> usize {
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => {
                // `\` + CRLF is a single LineContinuation sequence; every other
                // escape is two bytes. (`\` + LS/PS also continues the line: the
                // two-byte skip consumes the E2 lead byte and the remaining
                // bytes are ordinary string content.)
                if bytes.get(i + 1) == Some(&b'\r') && bytes.get(i + 2) == Some(&b'\n') {
                    i = i.saturating_add(3);
                } else {
                    i = i.saturating_add(2);
                }
            }
            // An unescaped LF or CR ends a (mal)formed string literal for the
            // lexer. Scanning past it to a later quote swallowed real source,
            // hiding brackets and type angles from the depth guard (#3213).
            // Unescaped LS/PS stay legal inside string literals (ES2019), so
            // only LF and CR terminate.
            b'\n' | b'\r' => return i,
            b if b == quote => return i + 1,
            _ => i += 1,
        }
    }
    i
}

/// Skip literal template text, returning the byte after either the closing
/// backtick or an opening `${`. The caller scans interpolation bodies so deeply
/// nested input is guarded without recursive Rust calls.
pub(super) fn skip_template_text(bytes: &[u8], mut i: usize) -> (usize, bool) {
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i = i.saturating_add(2),
            b'`' => return (i + 1, false),
            b'$' if bytes.get(i + 1) == Some(&b'{') => return (i + 2, true),
            _ => i += 1,
        }
    }
    (i, false)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SpeculativeTypeAngleOpen {
    Regular,
    MalformedIdentifierEscape,
}

pub(super) fn speculative_type_angle_open_kind(
    content: &str,
    open: usize,
) -> Option<SpeculativeTypeAngleOpen> {
    // The `<` is ASCII, so `open + 1` is a valid char boundary.
    let after = &content[open + 1..];
    let marker = skip_type_angle_trivia(after);
    // `{`/`[` start structural types (#2944); `!` starts a JSDoc non-nullable
    // type (#3213); `(` starts a parenthesized type (#3277/#3279/#3281); and an
    // identifier starts a type reference — `f<T>`, the plainest type-argument
    // list there is (#3712). Each keeps OXC inside type-argument speculation, so
    // repeated occurrences make unclosed angles count toward the depth budget.
    //
    // A digit is deliberately excluded. `f<0>` is a valid numeric literal type,
    // but `a < 1` is overwhelmingly a comparison, and the identifier arm alone
    // already covers every reproducer shape seen so far.
    match after.as_bytes().get(marker) {
        // `\` starts a `\uXXXX` identifier escape, which OXC lexes as an
        // identifier start just like a bare letter.
        Some(b'{' | b'[' | b'!' | b'(' | b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'$') => {
            Some(SpeculativeTypeAngleOpen::Regular)
        }
        Some(b'\\') if starts_valid_identifier_escape(after.as_bytes(), marker) => {
            Some(SpeculativeTypeAngleOpen::Regular)
        }
        Some(b'\\') => Some(SpeculativeTypeAngleOpen::MalformedIdentifierEscape),
        // Trivia is already skipped, so a remaining non-ASCII lead byte begins a
        // Unicode identifier (`f<日本語>`) rather than whitespace.
        Some(byte) if !byte.is_ascii() => Some(SpeculativeTypeAngleOpen::Regular),
        Some(_) | None => None,
    }
}

fn starts_valid_identifier_escape(bytes: &[u8], marker: usize) -> bool {
    decode_identifier_escape(bytes, marker).is_some_and(is_identifier_start)
}

fn decode_identifier_escape(bytes: &[u8], marker: usize) -> Option<char> {
    if bytes.get(marker) != Some(&b'\\') || bytes.get(marker + 1) != Some(&b'u') {
        return None;
    }
    if bytes.get(marker + 2) == Some(&b'{') {
        decode_braced_identifier_escape(bytes, marker + 3)
    } else {
        decode_fixed_identifier_escape(bytes, marker + 2)
    }
}

fn decode_braced_identifier_escape(bytes: &[u8], mut i: usize) -> Option<char> {
    let mut code_point = 0u32;
    let mut digits = 0usize;
    while digits < 6 {
        let Some(value) = bytes.get(i).and_then(|byte| hex_value(*byte)) else {
            break;
        };
        code_point = (code_point << 4) | value;
        i += 1;
        digits += 1;
    }
    if digits == 0 || bytes.get(i) != Some(&b'}') {
        return None;
    }
    char::from_u32(code_point)
}

fn decode_fixed_identifier_escape(bytes: &[u8], start: usize) -> Option<char> {
    let mut code_point = 0u32;
    for byte in bytes.get(start..start + 4)? {
        code_point = (code_point << 4) | hex_value(*byte)?;
    }
    char::from_u32(code_point)
}

fn hex_value(byte: u8) -> Option<u32> {
    match byte {
        b'0'..=b'9' => Some(u32::from(byte - b'0')),
        b'a'..=b'f' => Some(u32::from(byte - b'a' + 10)),
        b'A'..=b'F' => Some(u32::from(byte - b'A' + 10)),
        _ => None,
    }
}

/// Skip the lexer trivia OXC drops between `<` and the next token: ASCII and
/// ECMAScript Unicode whitespace, line terminators, and line/block comments.
/// The byte scanner must skip exactly what the lexer skips, or a marker hidden
/// behind trivia (`a</* */!`, `a<\u{a0}!`) slips past the speculative-angle
/// guard while OXC still recurses into the overflow path (#3213).
fn skip_type_angle_trivia(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut i = 0;
    loop {
        match bytes.get(i) {
            Some(b'/') if bytes.get(i + 1) == Some(&b'/') => i = skip_line_comment(bytes, i + 2),
            Some(b'/') if bytes.get(i + 1) == Some(&b'*') => i = skip_block_comment(bytes, i + 2),
            Some(&b) if b < 0x80 => {
                if matches!(b, b' ' | b'\t' | 0x0b | 0x0c | b'\n' | b'\r') {
                    i += 1;
                } else {
                    return i;
                }
            }
            // A multi-byte lead is a char boundary; decode it to test for
            // ECMAScript Unicode whitespace (NBSP, LS/PS, the Zs category, ...).
            Some(_) => {
                let c = s[i..].chars().next().unwrap();
                if is_ecmascript_whitespace(c) {
                    i += c.len_utf8();
                } else {
                    return i;
                }
            }
            None => return i,
        }
    }
}

/// ECMAScript WhiteSpace plus LineTerminator: the Unicode White_Space set minus
/// NEL (U+0085, which ECMAScript does not treat as whitespace) plus ZWNBSP
/// (U+FEFF, which is ECMAScript whitespace but lacks the White_Space property).
fn is_ecmascript_whitespace(c: char) -> bool {
    c == '\u{feff}' || (c.is_whitespace() && c != '\u{0085}')
}

/// Returns true when `content` contains no live tokens after an expression.
///
/// This is intentionally narrower than all ECMAScript trivia: trailing line
/// comments are valid JS trivia, but raw expression emitters often append their
/// own delimiters on the same generated line. Admitting a line comment here
/// would let authored source swallow those generated tokens.
pub fn is_expression_trailing_trivia(content: &str) -> bool {
    let bytes = content.as_bytes();
    let mut i = 0;
    loop {
        match bytes.get(i) {
            Some(b'/') if bytes.get(i + 1) == Some(&b'*') => {
                let Some(next) = skip_closed_block_comment(bytes, i + 2) else {
                    return false;
                };
                i = next;
            }
            Some(&b) if b < 0x80 => {
                if matches!(b, b' ' | b'\t' | 0x0b | 0x0c | b'\n' | b'\r') {
                    i += 1;
                } else {
                    return false;
                }
            }
            Some(_) => {
                if !content.is_char_boundary(i) {
                    return false;
                }
                let Some(c) = content[i..].chars().next() else {
                    return false;
                };
                if is_ecmascript_whitespace(c) {
                    i += c.len_utf8();
                } else {
                    return false;
                }
            }
            None => return true,
        }
    }
}

pub fn skip_line_comment(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() {
        // Line comments end at any ECMAScript line terminator: LF, CR, LS
        // (U+2028), or PS (U+2029). Stopping only at LF let a bare CR hide
        // parsed code from the guard (#3185).
        match bytes[i] {
            b'\n' | b'\r' => break,
            0xe2 if bytes.get(i + 1) == Some(&0x80)
                && matches!(bytes.get(i + 2), Some(&0xa8) | Some(&0xa9)) =>
            {
                break;
            }
            _ => i += 1,
        }
    }
    i
}

fn skip_closed_block_comment(bytes: &[u8], mut i: usize) -> Option<usize> {
    while i + 1 < bytes.len() {
        if bytes[i] == b'*' && bytes[i + 1] == b'/' {
            return Some(i + 2);
        }
        i += 1;
    }
    None
}

pub(super) fn skip_block_comment(bytes: &[u8], mut i: usize) -> usize {
    while i + 1 < bytes.len() {
        if bytes[i] == b'*' && bytes[i + 1] == b'/' {
            return i + 2;
        }
        i += 1;
    }
    bytes.len()
}

/// Skip a regex literal, returning where it ends.
///
/// Only a closing `/` terminates one. `None` means the literal never closed —
/// the scan hit a line terminator or the end of the input — and the lexer
/// reports that as an unterminated regex and recovers, leaving those bytes live
/// for the parser. Treating them as skipped literal text hid the source in
/// between from the depth budget: an unclosed `/` swallowed to EOF hid 183 type
/// angles (#3873), and one closed by a line terminator 27 KiB later hid 6182
/// more (#3875). The caller scans them instead, which can only over-count.
pub fn skip_regex(bytes: &[u8], mut i: usize) -> Option<usize> {
    let mut in_character_class = false;
    while i < bytes.len() {
        match bytes[i] {
            // A regex literal cannot span a line terminator, and `\` before one
            // is not a valid escape: the lexer ends the regex at the terminator.
            // Blindly skipping two bytes would swallow the terminator (LF/CR) or
            // its 0xE2 lead byte (LS/PS), hiding the following source from the
            // guard, so bail before consuming it.
            b'\\' => match bytes.get(i + 1) {
                Some(&b'\n' | &b'\r') => return None,
                Some(&0xe2)
                    if bytes.get(i + 2) == Some(&0x80)
                        && matches!(bytes.get(i + 3), Some(&0xa8) | Some(&0xa9)) =>
                {
                    return None;
                }
                _ => i = i.saturating_add(2),
            },
            b'[' => {
                in_character_class = true;
                i += 1;
            }
            b']' => {
                in_character_class = false;
                i += 1;
            }
            b'/' if !in_character_class => return Some(skip_identifier(bytes, i + 1)),
            b'\n' | b'\r' => return None,
            // LS/PS terminate an (unterminated) regex literal like LF/CR.
            0xe2 if bytes.get(i + 1) == Some(&0x80)
                && matches!(bytes.get(i + 2), Some(&0xa8) | Some(&0xa9)) =>
            {
                return None;
            }
            _ => i += 1,
        }
    }
    None
}

pub fn skip_identifier(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len()
        && matches!(bytes[i], b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'$')
    {
        i += 1;
    }
    i
}

pub fn skip_number(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len()
        && matches!(bytes[i], b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'.')
    {
        i += 1;
    }
    i
}

pub fn keyword_allows_regex_after(identifier: &[u8]) -> bool {
    matches!(
        identifier,
        b"await"
            | b"case"
            | b"delete"
            | b"do"
            | b"else"
            | b"in"
            | b"instanceof"
            | b"new"
            | b"of"
            | b"return"
            | b"throw"
            | b"typeof"
            | b"void"
            | b"yield"
    )
}
