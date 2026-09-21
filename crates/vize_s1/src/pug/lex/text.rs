//! Text lexing: inline and piped text, raw HTML lines, comments, pipeless
//! text blocks, and pug's `addText` splitter (`#[…]` tag interpolation,
//! `#{…}`/`!{…}` code interpolation, `\`-escapes).

use vize_s0::Vec;

use super::{Lexer, Tk};
use crate::pug::chars::parse_until;
use crate::pug::error::PugErrorCode;
use crate::pug::logical::{is_js_whitespace, skip_units};

impl Lexer<'_, '_> {
    /// `text`: `/^(?:\| ?| )([^\n]+)/ || /^( )/ || /^\|( ?)/`.
    pub(super) fn text(&mut self) -> bool {
        let input = self.input();
        let line = &input[..input.find('\n').unwrap_or(input.len())];
        let start = self.pos;
        let value = if let Some(rest) = line.strip_prefix('|') {
            self.push(Tk::Pipe, start, start + 1);
            // `\| ?` then `[^\n]+`: the optional space backtracks away
            // when it is the only byte left.
            let skip = usize::from(rest.len() >= 2 && rest.starts_with(' '));
            (start + 1 + skip, start + line.len())
        } else if line.starts_with(' ') {
            if line.len() >= 2 {
                (start + 1, start + line.len())
            } else {
                (start, start + 1)
            }
        } else {
            return false;
        };
        self.pos = value.1;
        self.add_text(Tk::Text, value.0, value.1);
        true
    }

    /// `textHtml`: `/^(<[^\n]*)/`.
    pub(super) fn text_html(&mut self) -> bool {
        if !self.input().starts_with('<') {
            return false;
        }
        let (start, end) = (self.pos, self.line_end());
        self.pos = end;
        self.add_text(Tk::TextHtml, start, end);
        true
    }

    /// `comment`: `/^\/\/(-)?([^\n]*)/`, then a pipeless body.
    pub(super) fn comment(&mut self) -> bool {
        if !self.input().starts_with("//") {
            return false;
        }
        let buffered = !self.input()[2..].starts_with('-');
        let marker_end = self.pos + if buffered { 2 } else { 3 };
        let end = self.line_end();
        self.push(Tk::Comment, self.pos, marker_end);
        self.push(Tk::CommentText, marker_end, end);
        self.pos = end;
        self.interpolation_allowed = buffered;
        self.pipeless_text(None);
        true
    }

    /// `pipelessText`: the indented block after `.`, a comment, a filter
    /// or block code — every line verbatim past the block's indentation.
    pub(super) fn pipeless_text(&mut self, indents: Option<usize>) -> bool {
        let mut retry = indents;
        let (lines, ptr) = 'lex: loop {
            while self.blank() {}
            let captures = self.scan_indentation().map(|(width, _)| width);
            let indents = match (retry, captures) {
                (Some(width), _) if width != 0 => width,
                (_, Some(width)) => width,
                _ => return false,
            };
            if indents <= self.top() {
                return false;
            }
            let tabs = self.indent_re == Some(super::IndentRe::Tabs);
            let indent_char = if tabs { b'\t' } else { b' ' };
            let mut lines: Vec<(usize, usize, usize)> = Vec::new_in(&self.allocator);
            let mut ptr = self.pos;
            loop {
                // `ptr` sits on the `\n` that precedes the candidate line.
                let line_start = ptr + 1;
                let rest = &self.src[line_start.min(self.end)..self.end];
                let line = &rest[..rest.find('\n').unwrap_or(rest.len())];
                let line_indents = line.bytes().take_while(|&b| b == indent_char).count();
                let blank = line.chars().all(is_js_whitespace);
                if line_indents >= indents || blank {
                    let value_start = line_start + skip_units(line, indents);
                    lines.push((line_start, value_start, line_start + line.len()));
                    ptr = line_start + line.len();
                } else if line_indents > self.top() {
                    // pug retries the whole block at the shallower width.
                    retry = Some(line_indents);
                    continue 'lex;
                } else {
                    break;
                }
                if ptr >= self.end {
                    break;
                }
            }
            break (lines, ptr);
        };
        let mut lines = lines;
        self.push(Tk::PipelessStart, self.pos, self.pos);
        self.pos = ptr;
        if self.pos >= self.end {
            while lines.last().is_some_and(|&(_, from, to)| from == to) {
                lines.pop();
            }
        }
        for (index, &(line_start, from, to)) in lines.iter().enumerate() {
            if index != 0 {
                self.push(Tk::Newline, line_start, line_start);
            }
            self.add_text(Tk::Text, from, to);
        }
        self.push(Tk::PipelessEnd, self.pos, self.pos);
        true
    }

    /// `addText` over the value `[start, end)`.
    pub(super) fn add_text(&mut self, kind: Tk, mut start: usize, end: usize) {
        let mut token_start = start;
        let mut escapes: Vec<usize> = Vec::new_in(&self.allocator);
        let mut has_prefix = false;
        loop {
            let value = &self.src[start..end];
            let none = usize::MAX;
            let at_end = if self.interpolated {
                value.find(']')
            } else {
                None
            };
            let at_end = at_end.unwrap_or(none);
            let allowed = self.interpolation_allowed;
            let at_tag = allowed.then(|| value.find("#[")).flatten().unwrap_or(none);
            let at_escape = allowed
                .then(|| value.find("\\#["))
                .flatten()
                .unwrap_or(none);
            let code = allowed.then(|| find_code_interpolation(value)).flatten();
            let at_code = code.map_or(none, |(at, _)| at);
            if at_escape < at_end && at_escape < at_tag && at_escape < at_code {
                escapes.push(start + at_escape);
                start += at_escape + 3;
                has_prefix = true;
                continue;
            }
            if at_tag < at_end && at_tag < at_escape && at_tag < at_code {
                self.emit_text(kind, token_start, start + at_tag, &escapes);
                let open = start + at_tag;
                self.push(Tk::InterpOpen, open, open + 2);
                let resume = self.child(open + 2, end);
                start = resume;
                token_start = resume;
                escapes.clear();
                has_prefix = false;
                continue;
            }
            if at_end < at_tag && at_end < at_escape && at_end < at_code {
                if has_prefix || at_end > 0 {
                    self.emit_text(kind, token_start, start + at_end, &escapes);
                }
                self.push(Tk::InterpClose, start + at_end, start + at_end + 1);
                self.pos = start + at_end + 1;
                self.ended = true;
                return;
            }
            if let Some((at, escaped)) = code {
                if escaped {
                    escapes.push(start + at);
                    start += at + 3;
                    has_prefix = true;
                    continue;
                }
                if has_prefix || at > 0 {
                    self.emit_text(kind, token_start, start + at, &escapes);
                }
                let body = start + at + 2;
                match parse_until(self.allocator, &self.src[body..end], b'}', 0) {
                    Ok(close) => {
                        self.push(Tk::CodeInterp, start + at, body + close + 1);
                        if body + close + 1 < end {
                            start = body + close + 1;
                            token_start = start;
                            escapes.clear();
                            has_prefix = false;
                            continue;
                        }
                    }
                    Err(_) => {
                        self.error(PugErrorCode::NoEndBracket, start + at);
                        self.push(Tk::CodeInterp, start + at, end);
                    }
                }
                return;
            }
            self.emit_text(kind, token_start, end, &escapes);
            return;
        }
    }

    /// One pug text token: its bytes with each escape backslash split out.
    fn emit_text(&mut self, kind: Tk, start: usize, end: usize, escapes: &[usize]) {
        let mut from = start;
        for &escape in escapes {
            self.push(kind, from, escape);
            self.push(Tk::Escape, escape, escape + 1);
            from = escape + 1;
        }
        self.push(kind, from, end);
    }

    /// Lex a `#[…]` body with a child lexer; returns where it stopped.
    fn child(&mut self, start: usize, end: usize) -> usize {
        let mut child = Lexer::new(
            self.allocator,
            self.src,
            start,
            end,
            self.out,
            self.errors,
            true,
        );
        vize_s0::ensure_sufficient_stack(|| child.run());
        child.pos
    }
}

/// The leftmost `(\\)?([#!]){` — pug's string-interpolation pattern.
fn find_code_interpolation(value: &str) -> Option<(usize, bool)> {
    let bytes = value.as_bytes();
    (0..bytes.len()).find_map(|at| {
        let opens = |from: usize| {
            matches!(bytes.get(from), Some(b'#' | b'!')) && bytes.get(from + 1) == Some(&b'{')
        };
        if bytes[at] == b'\\' && opens(at + 1) {
            Some((at, true))
        } else if opens(at) {
            Some((at, false))
        } else {
            None
        }
    })
}
