//! The *logical* pug text: what the pinned `pug` lexer actually reads.
//!
//! Vue's SFC parser hands pug the template content **dedented**
//! (`@vue/compiler-sfc`'s `dedent`: the smallest leading-whitespace run
//! over the non-blank lines is sliced off every line, blank lines
//! included), and pug's lexer then strips a leading BOM and normalizes
//! `\r\n` / lone `\r` to `\n`. The lexer below runs over that exact text so
//! its decisions are pug's; every logical byte keeps the original byte
//! offset it came from, so tokens map back onto the authored source and
//! the bytes the view dropped (the dedent prefix, the `\r` of a CRLF) fall
//! into the next token's `leading` slice — the S1 byte-fidelity law holds
//! over the authored bytes, not over the view.

use vize_s0::{Allocator, Vec};

/// The dedented, newline-normalized text with its origin map.
pub(crate) struct Logical<'a> {
    pub(crate) text: &'a str,
    /// `origin[i]` is the authored byte offset of logical byte `i`.
    origin: Vec<'a, u32>,
    source_len: u32,
}

impl<'a> Logical<'a> {
    pub(crate) fn new(allocator: &'a Allocator, source: &'a str) -> Self {
        let min_indent = min_indent(source);
        let mut text = vize_s0::StringBuilder::with_capacity_in(source.len(), allocator);
        let mut origin: Vec<'a, u32> = Vec::with_capacity_in(source.len(), &allocator);
        let mut line_start = 0usize;
        for line in source.split('\n') {
            if line_start != 0 {
                // The `\n` that joined this line to the previous one.
                push(&mut text, &mut origin, "\n", line_start - 1);
            }
            let skip = if min_indent == 0 {
                0
            } else {
                skip_units(line, min_indent)
            };
            let mut body_start = line_start + skip;
            let line_end = line_start + line.len();
            // pug strips one BOM off the start of the (dedented) text.
            if line_start == 0 && source[body_start..line_end].starts_with('\u{feff}') {
                body_start += '\u{feff}'.len_utf8();
            }
            let joined = line_end < source.len();
            let body = &source[body_start..line_end];
            push_normalized(&mut text, &mut origin, body, body_start, joined);
            line_start = line_end + 1;
        }
        Self {
            text: text.into_str(),
            origin,
            source_len: source.len() as u32,
        }
    }

    /// The authored offset of logical offset `at` (a zero-width position).
    pub(crate) fn at(&self, at: usize) -> u32 {
        self.origin.get(at).copied().unwrap_or(self.source_len)
    }

    /// The authored byte range of the logical range `[start, end)`.
    pub(crate) fn range(&self, start: usize, end: usize) -> (u32, u32) {
        let from = self.at(start);
        if end <= start {
            return (from, from);
        }
        let last = self.origin.get(end - 1).copied().unwrap_or(self.source_len);
        (from, (last + 1).min(self.source_len).max(from))
    }
}

fn push<'a>(
    text: &mut vize_s0::StringBuilder<'a>,
    origin: &mut Vec<'a, u32>,
    piece: &str,
    at: usize,
) {
    text.push_str(piece);
    for offset in 0..piece.len() {
        origin.push((at + offset) as u32);
    }
}

/// Copy one line body, turning every lone `\r` into `\n` and dropping the
/// `\r` of a `\r\n` pair (the joining `\n` carries that line break).
fn push_normalized<'a>(
    text: &mut vize_s0::StringBuilder<'a>,
    origin: &mut Vec<'a, u32>,
    body: &str,
    base: usize,
    joined: bool,
) {
    let bytes = body.as_bytes();
    let mut run = 0usize;
    for (index, &byte) in bytes.iter().enumerate() {
        if byte != b'\r' {
            continue;
        }
        push(text, origin, &body[run..index], base + run);
        if index + 1 < bytes.len() || !joined {
            push(text, origin, "\n", base + index);
        }
        run = index + 1;
    }
    push(text, origin, &body[run..], base + run);
}

/// `@vue/compiler-sfc`'s dedent width: the minimum count of leading
/// JavaScript-whitespace UTF-16 units over the lines that are not blank
/// under `String.prototype.trim` — `Infinity` (every line sliced empty)
/// when there is no such line.
fn min_indent(source: &str) -> usize {
    source
        .split('\n')
        .filter(|line| !line.chars().all(is_js_whitespace))
        .map(|line| {
            line.chars()
                .take_while(|&ch| is_js_whitespace(ch))
                .map(char::len_utf16)
                .sum::<usize>()
        })
        .min()
        .unwrap_or(usize::MAX)
}

/// Byte length of the prefix `String.prototype.slice(units)` removes.
pub(crate) fn skip_units(line: &str, units: usize) -> usize {
    let mut taken = 0usize;
    for (index, ch) in line.char_indices() {
        if taken >= units {
            return index;
        }
        taken += ch.len_utf16();
    }
    line.len()
}

/// ECMAScript `WhiteSpace` ∪ `LineTerminator` — the set `\s` and
/// `String.prototype.trim` agree on.
pub(crate) fn is_js_whitespace(ch: char) -> bool {
    matches!(
        ch,
        '\t' | '\n' | '\u{b}' | '\u{c}' | '\r' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200a}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202f}'
                | '\u{205f}'
                | '\u{3000}'
                | '\u{feff}'
    )
}
