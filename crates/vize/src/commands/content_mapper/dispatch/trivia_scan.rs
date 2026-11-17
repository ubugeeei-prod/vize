//! Where a projection's trivia is, without parsing it.
//!
//! A full parse (with tokens) costs the mapper about 30% more instructions,
//! so the compactor scans instead. The scan tracks only what
//! decides whether a byte is code: strings, template literals and their
//! substitutions, comments and regular expressions. Wherever that is not
//! certain it gives up, and the projection is sent unchanged:
//!
//! - a `/` after `)`, `}`, `>`, `/`, `++` or `--` may start a regular
//!   expression or divide; the scan does not guess;
//! - a projection of a TSX script (`/// <reference types="vue/jsx" />`) may
//!   hold JSX text, which only a parser tells apart from code, and so does
//!   anything with a closing tag (`</`);
//! - an unterminated string, comment, template or regular expression, a
//!   hashbang, or a Unicode line terminator (U+2028/U+2029) in a `//`
//!   comment also ends it. Elsewhere such a terminator is a code byte to the
//!   scan, so the trivia around it is never touched.

/// One comment's byte range and whether it is a `//` comment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Comment {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) line: bool,
}

/// What the previous code token says about a following `/`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Slash {
    Regex,
    Divide,
    Unknown,
}

/// What a `{` opened: a block or object, or a template substitution.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Brace {
    Plain,
    Substitution,
}

/// Keywords after which `/` starts a regular expression.
const REGEX_KEYWORDS: &[&[u8]] = &[
    b"return",
    b"typeof",
    b"instanceof",
    b"in",
    b"of",
    b"new",
    b"delete",
    b"void",
    b"throw",
    b"case",
    b"do",
    b"else",
    b"yield",
    b"await",
    b"default",
];

const VUE_JSX_REFERENCE: &str = "/// <reference types=\"vue/jsx\" />";

/// Hands each maximal run of ASCII whitespace and comments between code
/// bytes, in text order, to `on_trivia(start, end, comments)`; `None` when
/// the scan is unsure, whatever it handed out before. A lone whitespace byte
/// between two code bytes, where no comment or regular expression can start,
/// is not handed out: one-byte trivia without a comment is already as short
/// as it gets, and these are most of a projection's trivia runs.
pub(super) fn trivia(
    text: &str,
    mut on_trivia: impl FnMut(usize, usize, &[Comment]),
) -> Option<()> {
    if text.starts_with(VUE_JSX_REFERENCE) || text.starts_with("#!") {
        return None;
    }
    let bytes = text.as_bytes();
    let mut comments = Vec::new();
    let mut braces: Vec<Brace> = Vec::new();
    let mut at = 0;
    loop {
        // A closing JSX tag: its text is not code. `<` is plain code to the
        // run scan, so the tag is found at its slash, before that slash can
        // start a comment.
        if bytes.get(at) == Some(&b'/')
            && at.checked_sub(1).and_then(|index| bytes.get(index)) == Some(&b'<')
        {
            return None;
        }
        // Trivia: whitespace and comments.
        let start = at;
        comments.clear();
        loop {
            match (bytes.get(at), bytes.get(at + 1)) {
                (Some(b' ' | b'\t' | b'\n' | b'\r'), _) => at += 1,
                (Some(b'/'), Some(b'/')) => {
                    let end = line_comment_end(bytes, at)?;
                    comments.push(Comment {
                        start: at,
                        end,
                        line: true,
                    });
                    at = end;
                }
                (Some(b'/'), Some(b'*')) => {
                    let body = bytes.get(at + 2..)?;
                    let end = at + 2 + memchr::memmem::find(body, b"*/")? + 2;
                    comments.push(Comment {
                        start: at,
                        end,
                        line: false,
                    });
                    at = end;
                }
                _ => break,
            }
        }
        if at > start {
            on_trivia(start, at, &comments);
        }
        let Some(&byte) = bytes.get(at) else {
            return braces.is_empty().then_some(());
        };
        // One code token, or a run of ordinary code bytes.
        match byte {
            b'"' | b'\'' => at = string_end(bytes, at)?,
            b'`' => at = template_end(bytes, at + 1, &mut braces)?,
            // The code byte before the trivia just scanned decides.
            b'/' => match slash_after(bytes, start.checked_sub(1)) {
                Slash::Regex => at = regex_end(bytes, at)?,
                Slash::Divide => at += 1,
                Slash::Unknown => return None,
            },
            b'{' => {
                braces.push(Brace::Plain);
                at += 1;
            }
            b'}' => match braces.pop()? {
                Brace::Plain => at += 1,
                Brace::Substitution => at = template_end(bytes, at + 1, &mut braces)?,
            },
            // A closing JSX tag: its text is not code.
            b'<' if bytes.get(at + 1) == Some(&b'/') => return None,
            // A run of code; `vize_l0::code_run` steps over it sixteen
            // bytes at a time and crosses a lone whitespace byte.
            _ => at = vize_l0::code_run::skip_code(bytes, at + 1),
        }
    }
}

/// Whether a `/` after the code byte at `previous` starts a regular expression.
fn slash_after(bytes: &[u8], previous: Option<usize>) -> Slash {
    let Some(end) = previous else {
        return Slash::Regex;
    };
    let Some(&byte) = bytes.get(end) else {
        return Slash::Regex;
    };
    match byte {
        b')' | b'}' | b'/' | b'>' => Slash::Unknown,
        b']' | b'"' | b'\'' | b'`' => Slash::Divide,
        b'+' | b'-' if end.checked_sub(1).and_then(|index| bytes.get(index)) == Some(&byte) => {
            Slash::Unknown
        }
        _ if is_word(byte) => {
            let length = bytes.get(..=end).map_or(0, |head| {
                head.iter().rev().take_while(|byte| is_word(**byte)).count()
            });
            let word = bytes.get(end + 1 - length..=end).unwrap_or_default();
            if REGEX_KEYWORDS.contains(&word) {
                Slash::Regex
            } else {
                Slash::Divide
            }
        }
        _ => Slash::Regex,
    }
}

/// Where the line comment at `at` ends; `None` at a U+2028 or U+2029,
/// which ends it in JavaScript but not here.
fn line_comment_end(bytes: &[u8], at: usize) -> Option<usize> {
    let mut index = at;
    loop {
        let rest = bytes.get(index..)?;
        let Some(offset) = memchr::memchr3(b'\n', b'\r', 0xE2, rest) else {
            return Some(bytes.len());
        };
        index += offset;
        if bytes.get(index) != Some(&0xE2) {
            return Some(index);
        }
        if matches!(bytes.get(index + 1..index + 3), Some([0x80, 0xA8 | 0xA9])) {
            return None;
        }
        index += 1;
    }
}

/// Identifier, keyword and number bytes (any non-ASCII byte counts as one).
fn is_word(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$' | b'#' | b'.') || byte >= 0x80
}

/// The end of the string literal opening at `at`.
fn string_end(bytes: &[u8], at: usize) -> Option<usize> {
    let quote = *bytes.get(at)?;
    let mut index = at + 1;
    loop {
        match *bytes.get(index)? {
            byte if byte == quote => return Some(index + 1),
            b'\\' => index += 2,
            b'\n' | b'\r' => return None,
            _ => index += 1,
        }
    }
}

/// The end of template text starting at `at`: past its closing backtick, or
/// past a `${`, which pushes a substitution.
fn template_end(bytes: &[u8], mut at: usize, braces: &mut Vec<Brace>) -> Option<usize> {
    loop {
        match *bytes.get(at)? {
            b'\\' => at += 2,
            b'`' => return Some(at + 1),
            b'$' if bytes.get(at + 1) == Some(&b'{') => {
                braces.push(Brace::Substitution);
                return Some(at + 2);
            }
            _ => at += 1,
        }
    }
}

/// The end of the regular expression literal opening at `at`, flags included.
fn regex_end(bytes: &[u8], at: usize) -> Option<usize> {
    let mut index = at + 1;
    let mut class = false;
    loop {
        match *bytes.get(index)? {
            b'\\' => index += 2,
            b'\n' | b'\r' => return None,
            b'[' => {
                class = true;
                index += 1;
            }
            b']' => {
                class = false;
                index += 1;
            }
            b'/' if !class => {
                index += 1;
                break;
            }
            _ => index += 1,
        }
    }
    Some(
        index
            + bytes
                .get(index..)?
                .iter()
                .take_while(|byte| byte.is_ascii_alphanumeric())
                .count(),
    )
}

#[cfg(test)]
mod tests;
